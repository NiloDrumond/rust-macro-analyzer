import React from "react";
import {
  ResponsiveContainer,
  PieChart,
  Pie,
  Cell,
  Legend,
  Tooltip,
} from "recharts";
import { useData } from "../../hooks/use-data";
import { Card } from "../ui/card";
import { usageToCount } from "../../utils/data";

const COLORS = ["#0ea5e9", "#22c55e", "#f97316", "#8b5cf6"];

export function MacroInvocationCountByType() {
  const { data } = useData();

  const chartData = React.useMemo(() => {
    if (!data) return [];
    const total = data.total_macro_usage;
    return [
      {
        name: "Derive Macros",
        value: usageToCount(total.derive_macro_usage.derives),
      },
      {
        name: "Attribute Macros",
        value: usageToCount(total.attribute_macro_invocations),
      },
      {
        name: "Builtin Attribute Macros",
        value: usageToCount(total.builtin_attribute_macro_invocations),
      },
      {
        name: "Function-like and Declarative Macros",
        value: usageToCount(total.macro_invocations),
      },
    ];
  }, [data]);

  return (
    <Card>
      <h2>Macro Invocation Count By Type</h2>
      <ResponsiveContainer width="100%" height={400}>
        <PieChart>
          <Pie
            data={chartData}
            dataKey="value"
            nameKey="name"
            // cx="50%"
            // cy="50%"
            innerRadius={50}
            outerRadius={100}
            fill="#8884d8"
            label
          >
            {chartData.map((entry, index) => (
              <Cell
                key={`cell-${index}`}
                fill={COLORS[index % COLORS.length]}
                stroke={"#416981"}
              />
            ))}
          </Pie>
          <Legend />
          <Tooltip
            contentStyle={{
              background: "#1e293b",
              borderColor: "#64748b",
              borderRadius: 8,
            }}
            itemStyle={{ color: "#f1f5f9" }}
          />
        </PieChart>
      </ResponsiveContainer>
    </Card>
  );
}
