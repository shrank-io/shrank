import { BrowserRouter, Routes, Route } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import Sidebar from "./components/Layout/Sidebar";
import Header from "./components/Layout/Header";
import Dashboard from "./pages/Dashboard";
import Browse from "./pages/Browse";
import SearchPage from "./pages/SearchPage";
import DocumentPage from "./pages/DocumentPage";
import GraphPage from "./pages/GraphPage";
import UploadPage from "./pages/UploadPage";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 30_000,
      retry: 1,
    },
  },
});

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <div className="flex h-dvh">
          <Sidebar />
          <div className="ml-56 flex flex-1 flex-col overflow-hidden">
            <Header />
            <main className="flex-1 overflow-y-auto">
              <Routes>
                <Route path="/" element={<Dashboard />} />
                <Route path="/browse" element={<Browse />} />
                <Route path="/search" element={<SearchPage />} />
                <Route path="/documents/:id" element={<DocumentPage />} />
                <Route path="/graph" element={<GraphPage />} />
                <Route path="/upload" element={<UploadPage />} />
              </Routes>
            </main>
          </div>
        </div>
      </BrowserRouter>
    </QueryClientProvider>
  );
}
