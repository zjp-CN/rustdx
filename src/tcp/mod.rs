use crate::bytes_helper::u16_from_le_bytes;
use log::trace;
use std::{
    io::{BufReader, Read, Result, Write},
    net::{SocketAddr, TcpStream},
    time::Duration,
};

#[macro_use]
#[cfg(test)]
pub(crate) mod tests;

mod basic;
pub use basic::*;

pub mod helper;
pub mod ip;

pub mod stock;

/// 用于缓冲读取 TcpStream 的数据。
pub type BufTcp = BufReader<TcpStream>;

/// Tcp 链接的一层封装。
#[derive(Debug)]
pub struct Tcp {
    stream: TcpStream,
    buffer: BufTcp,
    recv: [u8; RECV_SIZE],
}

impl Tcp {
    /// 已发送三个测试包
    pub fn new() -> Result<Self> {
        let (stream, buffer, recv) = tcpstream()?;
        let mut tcp = Self {
            stream,
            buffer,
            recv,
        };
        send_packs(&mut tcp, false)?;
        Ok(tcp)
    }

    /// 已发送三个测试包
    pub fn new_with_ip(ip: &SocketAddr) -> Result<Self> {
        let (stream, buffer, recv) = tcpstream_ip(ip)?;
        let mut tcp = Self {
            stream,
            buffer,
            recv,
        };
        send_packs(&mut tcp, false)?;
        Ok(tcp)
    }

    /// 发送并接收字节。需要对接收的字节进行解析（参考 [`Tdx::parse`] 的实现）。
    ///
    /// 方法返回发送和读取的字节数。
    ///
    /// 注意是读取而不是接收的字节数。
    /// 由于每次接收先读取 16 字节，所以返回的元组中，第二个数字应为 16。
    pub fn send_recv(&mut self, send: &[u8]) -> Result<(usize, usize)> {
        Ok((self.stream.write(send)?, self.buffer.read(&mut self.recv)?))
    }

    pub fn into_inner(self) -> (TcpStream, BufTcp, [u8; RECV_SIZE]) {
        (self.stream, self.buffer, self.recv)
    }

    pub fn get_ref(&self) -> (&TcpStream, &BufTcp, &[u8]) {
        (&self.stream, &self.buffer, &self.recv)
    }

    pub fn get_ref_recv(&self) -> &[u8] {
        &self.recv
    }
}

pub trait Tdx {
    /// 待发送的字节。所有发送请求的字节由两部分组成：
    /// 1. 固定的默认字节（基本为前半段字节）
    /// 2. 查询所需的变化的字节（基本为后半段字节）
    ///
    /// [`Tdx::SEND`] 的作用为默认有效的字节，即发送字节之后会正常响应且数据可以解析，
    /// 但是并不保证每次响应的数据完全一致。
    /// 比如请求日线时发送字节在当天收盘后是不变的，次日交易日请求得到的数据则可能改变。
    ///
    /// 如果发送请求的字节有误，则无法得到响应
    /// （比如设置了读取超时，无响应情况下会得到：
    /// [`WouldBlock` error][`std::io::ErrorKind::WouldBlock`]）。
    ///
    /// 字节具体的含义见 Implementor 的 Tdx trait 部分的 `SEND` 文档。
    const SEND: &'static [u8];
    /// 描述此次 tcp 连接的用途，目前用于记录日志。
    const TAG: &'static str;
    /// 发送的字节的长度。每种请求所发送的字节长度是已知的。默认为 [`Tdx::SEND`] 的长度。
    const LEN: usize = Self::SEND.len();
    type Item: ?Sized;

    /// 真正发送的字节。
    fn send(&mut self) -> &[u8];

    /// 得到响应的字节。响应的字节长度无法预测。
    fn recv(&mut self, tcp: &mut Tcp) -> Result<Vec<u8>> {
        send_recv_decompress(tcp, self.send(), Self::TAG)
    }

    /// 解析响应的字节。
    fn parse(&mut self, response: Vec<u8>);

    /// 得到和解析响应的字节，并返回解析的数据。
    fn recv_parsed(&mut self, tcp: &mut Tcp) -> Result<&Self::Item> {
        let response = self.recv(tcp)?;
        self.parse(response);
        Ok(self.result())
    }

    /// 解析后的结果。可以是引用，也可以是 owned type 。
    fn result(&self) -> &Self::Item;
}

/// 此函数完成以下事情：
/// 1. 接收响应的字节，并且在 debug 模式下验证有效数据的长度
/// 2. 根据响应信息的解压前后长度，进行数据解压
/// 3. 消耗缓冲区的字节（否则下次 read 的内容是上次未读/未消耗的字节），返回有效数据
///
/// 有效数据：包含实际有用信息的数据。
pub fn send_recv_decompress(tcp: &mut Tcp, send: &[u8], tag: &str) -> Result<Vec<u8>> {
    let (mut buf, deflate_size, inflate_size) = send_recv(tcp, send, tag)?;

    if deflate_size != inflate_size {
        buf = miniz_oxide::inflate::decompress_to_vec_zlib(&buf).unwrap();
        trace!("解压后数据：\n{:?}\n", buf);
        debug_assert_eq!(buf.len(), inflate_size as usize);
    } else {
        trace!("无需解压\n");
    };

    Ok(buf)
}

// 由于只读取了前 16 字节（TCP_RECV_SIZE），
// 剩下的未读字节需要使用内部缓冲区消耗掉，
// 否则下次 read 的内容是上次的未读字节。
// 如果不使用 BufReader ，那么需要手动 read 剩余字节。
// 对于 TcpStream ，Write::flush 没有做任何事情，所以无需调用。
pub fn send_recv(tcp: &mut Tcp, send: &[u8], tag: &str) -> Result<(Vec<u8>, u16, u16)> {
    tcp.send_recv(send)?;
    trace!("{}\nsend: {:?}\nrecv[16B]: {:?}", tag, send, tcp);

    let deflate_size = u16_from_le_bytes(&tcp.recv, 12); // 响应信息中的待解压长度
    let mut buf = vec![0; deflate_size as usize];
    tcp.buffer.read_exact(&mut buf)?;

    let inflate_size = u16_from_le_bytes(&tcp.recv, 14); // 响应信息中的解压后长度
    #[rustfmt::skip]
    trace!("\n解压前：#{:?}# -> {}，解压后：#{:?}# -> {}\n剩余数据（即解压前）：{:x?}\n",
           &tcp.recv[12..14], deflate_size, &tcp.recv[14..16], inflate_size, buf);

    Ok((buf, deflate_size, inflate_size))
}

/// 默认的超时值。
///
/// 增加到5秒，以适应网络延迟和服务器的响应时间
pub const TIMEOUT: Duration = Duration::from_secs(5);

/// 快速引入 tcpstream，设置 5 秒超时。
/// TODO: 固定？随机？取最快的 ip？
pub fn tcpstream() -> Result<(TcpStream, BufTcp, [u8; RECV_SIZE])> {
    tcpstream_ip(&ip::STOCK_IP[0])
}

/// 快速引入 tcpstream，设置 5 秒超时。
pub fn tcpstream_ip(ip: &SocketAddr) -> Result<(TcpStream, BufTcp, [u8; RECV_SIZE])> {
    let stream = TcpStream::connect_timeout(ip, TIMEOUT)?;
    stream.set_read_timeout(Some(TIMEOUT))?;
    stream.set_write_timeout(Some(TIMEOUT))?;
    let recv = [0; RECV_SIZE];
    let buffer = BufReader::new(stream.try_clone()?);
    Ok((stream, buffer, recv))
}
