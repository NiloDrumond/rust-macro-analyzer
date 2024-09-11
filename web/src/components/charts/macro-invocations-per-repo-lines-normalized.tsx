import { ApexOptions } from "apexcharts";
import React from "react";
import ReactApexChart from "react-apexcharts";
import { useData } from "../../hooks/use-data";
import { getRepoName } from "../../utils/string";
import { Card } from "../ui/card";

export function MacroInvocationsPerRepoLinesNormalized() {
  const { data } = useData();

  const chartData = React.useMemo(() => {
    if (!data) return [];
    return data.macro_invocations_per_repo
      .map(([path, count]) => {
        const lines = data.lines_per_repo[path];
        const y = lines > 0 ? count / lines : 0;
        return {
          x: path,
          y,
        };
      })
      .sort((a, b) => b.y - a.y);
  }, [data]);

  const options = React.useMemo(() => {
    const options: ApexOptions = {
      legend: { show: false },
      theme: { mode: "dark" },
      dataLabels: {
        formatter: (val) => {
          return getRepoName(val as string);
        },
        style: {
          fontSize: "12px",
        },
      },
      plotOptions: {
        treemap: {
          shadeIntensity: 0.7,
          useFillColorAsStroke: true,
          dataLabels: { format: "truncate" },
          colorScale: {
            // max,
            // min,
            // ranges: [
            //   { from: min, to: max / 2, color: "#14b8a6" },
            //   { from: max / 2, to: max, color: "#dc2626" },
            // ],
          },
        },
      },
      chart: {
        height: 350,
        background: "transparent",
        type: "treemap",
        toolbar: {
          show: false,
        },
      },
    };
    return options;
  }, []);

  return (
    <Card>
      <h2>Macro Invocations Per Repository - Lines Normalized</h2>

      <ReactApexChart
        options={options}
        series={[{ data: chartData }]}
        height={800}
        type="treemap"
      />
    </Card>
  );
}
