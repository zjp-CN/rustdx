#!/usr/bin/env python3
"""
测试pytdx的get_minute_time_data功能
"""
from pytdx.hq import TdxHq_API

print("正在连接到通达信服务器...")
api = TdxHq_API()

if api.connect('115.238.56.198', 7709):
    print("✅ 连接成功！\n")

    # 测试获取深市股票分时数据
    print("=" * 60)
    print("测试: 获取000001平安银行的分时数据")
    print("=" * 60)

    data = api.get_minute_time_data(0, '000001')
    print(f"返回数量: {len(data) if data else 0}")

    if data and len(data) > 0:
        print(f"\n数据类型: {type(data[0])}")
        print("\n前10个数据点:")
        for i, item in enumerate(data[:10]):
            print(f"{i+1:2d}. 价格: {item.get('price', 'N/A'):7.2f}  成交量: {item.get('vol', 'N/A')}")

        print(f"\n最后5个数据点:")
        for i, item in enumerate(data[-5:]):
            idx = len(data) - 5 + i + 1
            print(f"{idx:2d}. 价格: {item.get('price', 'N/A'):7.2f}  成交量: {item.get('vol', 'N/A')}")

    # 测试获取沪市股票分时数据
    print("\n" + "=" * 60)
    print("测试: 获取600000浦发银行的分时数据")
    print("=" * 60)

    data = api.get_minute_time_data(1, '600000')
    print(f"返回数量: {len(data) if data else 0}")

    if data and len(data) > 0:
        print("\n前10个数据点:")
        for i, item in enumerate(data[:10]):
            print(f"{i+1:2d}. 价格: {item.get('price', 'N/A'):7.2f}  成交量: {item.get('vol', 'N/A')}")

    api.disconnect()
    print("\n✅ 测试完成！")
else:
    print("❌ 连接失败")
