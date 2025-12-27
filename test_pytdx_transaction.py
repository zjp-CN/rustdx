#!/usr/bin/env python3
"""
测试pytdx的get_transaction_data功能
"""
from pytdx.hq import TdxHq_API

print("正在连接到通达信服务器...")
api = TdxHq_API()

if api.connect('115.238.56.198', 7709):
    print("✅ 连接成功！\n")

    # 测试获取深市股票成交明细
    print("=" * 60)
    print("测试: 获取000001平安银行的成交明细（前20笔）")
    print("=" * 60)

    data = api.get_transaction_data(0, '000001', 0, 20)
    print(f"返回数量: {len(data) if data else 0}")

    if data and len(data) > 0:
        print(f"\n数据类型: {type(data[0])}")
        print(f"\n前20笔成交:")
        print("   时间      价格     成交量   编号   买卖")
        print("   " + "-" * 45)
        for i, item in enumerate(data[:20]):
            print(f"   {item.get('time', 'N/A')} "
                  f"{item.get('price', 'N/A'):7.2f} "
                  f"{item.get('vol', 'N/A'):6} "
                  f"{item.get('num', 'N/A'):4} "
                  f"{item.get('buyorsell', 'N/A')}")

    # 测试获取沪市股票成交明细
    print("\n" + "=" * 60)
    print("测试: 获取600000浦发银行的成交明细（前20笔）")
    print("=" * 60)

    data = api.get_transaction_data(1, '600000', 0, 20)
    print(f"返回数量: {len(data) if data else 0}")

    if data and len(data) > 0:
        print(f"\n前20笔成交:")
        print("   时间      价格     成交量   编号   买卖")
        print("   " + "-" * 45)
        for i, item in enumerate(data[:20]):
            print(f"   {item.get('time', 'N/A')} "
                  f"{item.get('price', 'N/A'):7.2f} "
                  f"{item.get('vol', 'N/A'):6} "
                  f"{item.get('num', 'N/A'):4} "
                  f"{item.get('buyorsell', 'N/A')}")

    # 测试获取更多成交明细
    print("\n" + "=" * 60)
    print("测试: 分页获取000001的成交明细（start=20, count=20）")
    print("=" * 60)

    data = api.get_transaction_data(0, '000001', 20, 20)
    print(f"返回数量: {len(data) if data else 0}")

    if data and len(data) > 0:
        print(f"\n前10笔成交:")
        print("   时间      价格     成交量   编号   买卖")
        print("   " + "-" * 45)
        for i, item in enumerate(data[:10]):
            print(f"   {item.get('time', 'N/A')} "
                  f"{item.get('price', 'N/A'):7.2f} "
                  f"{item.get('vol', 'N/A'):6} "
                  f"{item.get('num', 'N/A'):4} "
                  f"{item.get('buyorsell', 'N/A')}")

    api.disconnect()
    print("\n✅ 测试完成！")
else:
    print("❌ 连接失败")
