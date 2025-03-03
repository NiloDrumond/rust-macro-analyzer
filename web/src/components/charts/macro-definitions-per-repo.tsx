import { ApexOptions } from "apexcharts";
import React from "react";
import ReactApexChart from "react-apexcharts";
import { useData } from "../../hooks/use-data";
import { getRepoName } from "../../utils/string";
import { Card } from "../ui/card";

export function MacroDefinitionsPerRepo() {
  const { data } = useData();

  const chartData = React.useMemo(() => {
    if (!data) return [];
    return data.macro_definitions_per_repo
      .map(([x, y]) => ({
        x,
        y,
      }))
      .filter(({ y }) => y > 0)
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
        type: "treemap",
        background: "transparent",
        toolbar: {
          show: false,
        },
      },
    };
    return options;
  }, []);

  return (
    <Card>
      <h2>Macro Definitions Per Repository</h2>

      <ReactApexChart
        options={options}
        series={[{ data: chartData }]}
        height={800}
        type="treemap"
      />
    </Card>
  );
}
