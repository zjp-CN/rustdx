#!/usr/bin/env python3
"""
测试pytdx获取指数行情
"""
from pytdx.hq import TdxHq_API

print("正在连接到通达信服务器...")
api = TdxHq_API()

if api.connect('115.238.56.198', 7709):
    print("✅ 连接成功！\n")

    # 测试使用get_security_quotes获取指数行情
    print("=" * 60)
    print("测试: 使用get_security_quotes获取上证指数(000001)")
    print("=" * 60)

    # 注意：在SecurityList返回的列表中，指数代码是000001等
    # 让我们尝试获取指数行情
    data = api.get_security_quotes([(1, '000001')])  # 上证指数
    print(f"返回数量: {len(data) if data else 0}")

    if data and len(data) > 0:
        quote = data[0]
        print(f"\n数据类型: {type(quote)}")
        print(f"\n指数行情:")
        print(f"  代码: {quote.get('code', 'N/A')}")
        print(f"  名称: {quote.get('name', 'N/A')}")
        print(f"  当前价: {quote.get('price', 'N/A')}")
        print(f"  昨收: {quote.get('last_close', 'N/A')}")
        print(f"  今开: {quote.get('open', 'N/A')}")
        print(f"  最高: {quote.get('high', 'N/A')}")
        print(f"  最低: {quote.get('low', 'N/A')}")
        print(f"  成交量: {quote.get('vol', 'N/A')}")
        print(f"  成交额: {quote.get('amount', 'N/A')}")
        print(f"  涨跌幅: {quote.get('change_percent', 'N/A')}")

    # 测试深证成指
    print("\n" + "=" * 60)
    print("测试: 使用get_security_quotes获取深证成指(399001)")
    print("=" * 60)

    data = api.get_security_quotes([(0, '399001')])  # 深证成指
    print(f"返回数量: {len(data) if data else 0}")

    if data and len(data) > 0:
        quote = data[0]
        print(f"\n指数行情:")
        print(f"  代码: {quote.get('code', 'N/A')}")
        print(f"  名称: {quote.get('name', 'N/A')}")
        print(f"  当前价: {quote.get('price', 'N/A')}")
        print(f"  涨跌幅: {quote.get('change_percent', 'N/A')}")

    api.disconnect()
    print("\n✅ 测试完成！")
else:
    print("❌ 连接失败")
