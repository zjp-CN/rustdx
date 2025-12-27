#!/usr/bin/env python3
"""
测试pytdx的get_security_quotes获取普通股票
"""
from pytdx.hq import TdxHq_API

print("正在连接到通达信服务器...")
api = TdxHq_API()

if api.connect('115.238.56.198', 7709):
    print("✅ 连接成功！\n")

    # 测试获取普通股票
    print("=" * 80)
    print("测试: 获取000001平安银行(普通股票)")
    print("=" * 80)

    data = api.get_security_quotes([(0, '000001')])
    print(f"返回数量: {len(data) if data else 0}")

    if data and len(data) > 0:
        quote = data[0]
        print(f"\n股票行情:")
        print(f"  代码: {quote.get('code', 'N/A')}")
        print(f"  名称: {quote.get('name', 'N/A')}")
        print(f"  当前价: {quote.get('price', 'N/A')}")
        print(f"  昨收: {quote.get('last_close', 'N/A')}")

    api.disconnect()
    print("\n✅ 测试完成！")
else:
    print("❌ 连接失败")
