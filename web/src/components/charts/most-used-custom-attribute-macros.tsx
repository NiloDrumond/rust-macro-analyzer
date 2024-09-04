import { ApexOptions } from "apexcharts";
import React from "react";
import ReactApexChart from "react-apexcharts";
import { useData } from "../../hooks/use-data";
import { Card } from "../ui/card";

export function MostUsedCustomAttributeMacros() {
  const { data } = useData();

  const chartData = React.useMemo(() => {
    if (!data) return [];

    return Object.entries(
      data.total_macro_usage.attribute_macro_invocations,
    )
      .map(([x, y]) => ({ x, y }))
      .sort((a, b) => b.y - a.y)
      .slice(0, 20);
  }, [data]);

  const options = React.useMemo(() => {
    const options: ApexOptions = {
      legend: { show: false },
      theme: { mode: "dark" },
      tooltip: {
        enabled: false,
      },
      chart: {
        height: 350,
        background: "transparent",
        type: "bar",
        toolbar: {
          show: false,
        },
      },
    };
    return options;
  }, []);

  return (
    <Card className="col-span-2">
      <h2>Most Used Custom Attribute Macros (Top 20)</h2>
      <ReactApexChart
        options={options}
        series={[{ data: chartData }]}
        type="bar"
        height={350}
      />
    </Card>
  );
}
