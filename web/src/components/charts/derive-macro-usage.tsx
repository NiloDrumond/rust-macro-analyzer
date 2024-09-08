import {
  ResponsiveContainer,
  LineChart, XAxis,
  YAxis,
  Tooltip,
  Legend,
  Line
} from "recharts";
import { useData } from "../../hooks/use-data";
import { Card } from "../ui/card";
import React from "react";

const countOccurrences = (sortedArray: number[]): number[] => {
  const counts: number[] = [];
  let currentCount = 1;

  for (let i = 1; i <= sortedArray.length; i++) {
    if (sortedArray[i] === sortedArray[i - 1]) {
      currentCount++;
    } else {
      counts.push(currentCount);
      currentCount = 1; // Reset for the next distinct number
    }
  }

  return counts;
};

export function DeriveMacroUsage() {
  const { data } = useData();

  const chartData = React.useMemo(() => {
    if (!data) return [];
    const occurrences = countOccurrences(data.derive_usage.sorted_data);
    return occurrences.map((v, index) => ({
      count: v,
      name: `${index + 1} derives`,
    }));
  }, [data]);

  return (
    <Card>
      <h2>Derives Per #[derive(...)] Attribute</h2>
      <ResponsiveContainer width="100%" height={300}>
        <LineChart data={chartData}>
          <XAxis dataKey="name" />
          <YAxis />
          <Tooltip
            contentStyle={{
              background: "#1e293b",
              borderColor: "#64748b",
              borderRadius: 8,
            }}
            itemStyle={{ color: "#f1f5f9" }}
          />
          <Legend />
          <Line
            type="monotone"
            dataKey="count"
            stroke="#8884d8"
            activeDot={{ r: 8 }}
          />
        </LineChart>
      </ResponsiveContainer>
      <div className="flex flex-row gap-4">
        <div>
          <strong>Average: </strong>
          {data?.derive_usage.avg}
        </div>
        <div>
          <strong>Median: </strong>
          {data?.derive_usage.median}
        </div>
        <div>
          <strong>Mode: </strong>
          {data?.derive_usage.mode}
        </div>
        <div>
          <strong>Max: </strong>
          {data?.derive_usage.max}
        </div>
      </div>
    </Card>
  );
}
