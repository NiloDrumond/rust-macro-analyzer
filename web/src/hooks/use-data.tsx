import { useQuery } from "@tanstack/react-query";
import React from "react";
import { createContext } from "../utils/react-utils";
import { Data } from "../../bindings/Data";

async function fetchData() {
  const response = await fetch("http://127.0.0.1:8080/data", { method: "GET" });
  if (!response.body) throw new Error();
  const data: Data = await response.json();
  return data;
}

function useDataController() {
  const { data } = useQuery({
    queryKey: ["data"],
    queryFn: fetchData,
    gcTime: Infinity,
    staleTime: Infinity,
  });
  console.log(data);

  return { data };
}

type DataController = ReturnType<typeof useDataController>;

const [Provider, useData] = createContext<DataController>();

export function DataProvider({ children }: React.PropsWithChildren) {
  const ctx = useDataController();
  return <Provider value={ctx}> {children} </Provider>;
}

export { useData };
