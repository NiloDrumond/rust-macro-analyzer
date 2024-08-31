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

const COLORS = ["#0ea5e9", "#22c55e", "#f97316", "#8b5cf6"];

export function MacroDefinitionCountByType() {
  const { data } = useData();

  const chartData = React.useMemo(() => {
    if(!data) return [];
    const {
      derive_macro_definitions,
      procedural_macro_definitions,
      attribute_macro_definitions,
      declarative_macro_definitions,
    } = data.total_macro_usage;
    return [
      {
        name: "Derive Macros",
        value: Object.values(derive_macro_definitions).reduce((a, b) => a + b),
      },
      {
        name: "Attribute Macros",
        value: Object.values(attribute_macro_definitions).reduce(
          (a, b) => a + b,
        ),
      },
      {
        name: "Declarative Macros",
        value: Object.values(declarative_macro_definitions).reduce(
          (a, b) => a + b,
        ),
      },
      {
        name: "Function-like Macros",
        value: Object.values(procedural_macro_definitions).reduce(
          (a, b) => a + b,
        ),
      },
    ];
  }, [data]);

  return (
    <Card>
      <h2>Macro Definition Count By Type</h2>
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
