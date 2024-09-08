import { useData } from "../../hooks/use-data";
import { Card } from "../ui/card";

export function MiscInfo() {
  const { data } = useData();
  return (
    <Card>
      <h4>Derives used in a single derive attribute:</h4>
      <ul>
        <li>
          <strong>Average: </strong>
          {data?.derive_usage.avg} (Oddly close to PI for some reason)
        </li>
        <li>
          <strong>Median: </strong>
          {data?.derive_usage.median}
        </li>
        <li>
          <strong>Mode: </strong>
          {data?.derive_usage.mode}
        </li>
        <li>
          <strong>Max: </strong>
          {data?.derive_usage.max}
        </li>
      </ul>
    </Card>
  );
}
