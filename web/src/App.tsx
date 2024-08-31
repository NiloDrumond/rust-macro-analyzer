import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { DataProvider } from "./hooks/use-data";
import { Results } from "./components/result";

const queryClient = new QueryClient();

function App() {
  return (
    <main className="w-screen h-screen">
      <QueryClientProvider client={queryClient}>
        <DataProvider>
          <Results />
        </DataProvider>
      </QueryClientProvider>
    </main>
  );
}

export default App;
