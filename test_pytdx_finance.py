#!/usr/bin/env python3
"""
测试pytdx的get_finance_info功能
"""
from pytdx.hq import TdxHq_API

print("正在连接到通达信服务器...")
api = TdxHq_API()

if api.connect('115.238.56.198', 7709):
    print("✅ 连接成功！\n")

    # 测试获取深市股票财务信息
    print("=" * 80)
    print("测试: 获取000001平安银行的财务信息")
    print("=" * 80)

    data = api.get_finance_info(0, '000001')
    print(f"返回类型: {type(data)}")

    if data:
        print(f"\n股票代码: {data.get('code', 'N/A')}")
        print(f"市场: {data.get('market', 'N/A')}")
        print(f"\n股本信息:")
        print(f"  流通股本: {data.get('liutongguben', 'N/A'):,.0f} 股")
        print(f"  总股本: {data.get('zongguben', 'N/A'):,.0f} 股")
        print(f"\n基本信息:")
        print(f"  所属省份: {data.get('province', 'N/A')}")
        print(f"  所属行业: {data.get('industry', 'N/A')}")
        print(f"  上市日期: {data.get('ipo_date', 'N/A')}")
        print(f"  更新日期: {data.get('updated_date', 'N/A')}")
        print(f"\n财务指标:")
        print(f"  总资产: {data.get('zongzichan', 'N/A'):,.0f} 元")
        print(f"  流动资产: {data.get('liudongzichan', 'N/A'):,.0f} 元")
        print(f"  固定资产: {data.get('gudingzichan', 'N/A'):,.0f} 元")
        print(f"  净资产: {data.get('jingzichan', 'N/A'):,.0f} 元")
        print(f"  主营收入: {data.get('zhuyingshouru', 'N/A'):,.0f} 元")
        print(f"  净利润: {data.get('jinglirun', 'N/A'):,.0f} 元")

    # 测试获取沪市股票财务信息
    print("\n" + "=" * 80)
    print("测试: 获取600000浦发银行的财务信息")
    print("=" * 80)

    data = api.get_finance_info(1, '600000')
    print(f"返回类型: {type(data)}")

    if data:
        print(f"\n股票代码: {data.get('code', 'N/A')}")
        print(f"  流通股本: {data.get('liutongguben', 'N/A'):,.0f} 股")
        print(f"  总股本: {data.get('zongguben', 'N/A'):,.0f} 股")
        print(f"  上市日期: {data.get('ipo_date', 'N/A')}")
        print(f"  总资产: {data.get('zongzichan', 'N/A'):,.0f} 元")
        print(f"  净资产: {data.get('jingzichan', 'N/A'):,.0f} 元")

    api.disconnect()
    print("\n✅ 测试完成！")
else:
    print("❌ 连接失败")
