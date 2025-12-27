#!/usr/bin/env python3
"""
测试pytdx的get_security_list功能
"""
from pytdx.hq import TdxHq_API

print("正在连接到通达信服务器...")
api = TdxHq_API()

if api.connect('115.238.56.198', 7709):
    print("✅ 连接成功！\n")

    # 测试获取深市股票列表
    print("=" * 60)
    print("测试1: 获取深市股票列表 (market=0, start=0)")
    print("=" * 60)

    data = api.get_security_list(0, 0)
    print(f"返回数量: {len(data)}")

    if data and len(data) > 0:
        print("\n前5只股票:")
        for i, stock in enumerate(data[:5]):
            print(f"\n{i+1}. {stock.get('code', 'N/A')} - {stock.get('name', 'N/A')}")
            print(f"   成交量单位: {stock.get('volunit', 'N/A')}")
            print(f"   小数点位: {stock.get('decimal_point', 'N/A')}")
            print(f"   昨收价: {stock.get('pre_close', 'N/A')}")

    # 测试获取沪市股票列表
    print("\n" + "=" * 60)
    print("测试2: 获取沪市股票列表 (market=1, start=0)")
    print("=" * 60)

    data = api.get_security_list(1, 0)
    if data is not None:
        print(f"返回数量: {len(data)}")

        if len(data) > 0:
            print("\n前5只股票:")
            for i, stock in enumerate(data[:5]):
                print(f"\n{i+1}. {stock.get('code', 'N/A')} - {stock.get('name', 'N/A')}")
                print(f"   成交量单位: {stock.get('volunit', 'N/A')}")
                print(f"   小数点位: {stock.get('decimal_point', 'N/A')}")
    else:
        print("⚠️ 沪市数据返回为空")

    # 测试分页获取深市股票
    print("\n" + "=" * 60)
    print("测试3: 分页获取深市股票 (market=0, start=1000)")
    print("=" * 60)

    data = api.get_security_list(0, 1000)
    if data is not None:
        print(f"返回数量: {len(data)}")

        if len(data) > 0:
            print("\n前5只股票:")
            for i, stock in enumerate(data[:5]):
                print(f"\n{i+1}. {stock.get('code', 'N/A')} - {stock.get('name', 'N/A')}")
                print(f"   成交量单位: {stock.get('volunit', 'N/A')}")

    api.disconnect()
    print("\n✅ 测试完成！")
else:
    print("❌ 连接失败")
