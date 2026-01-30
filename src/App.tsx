import { MainLayout } from "./components/common/MainLayout";
import { ContentArea } from "./components/common/ContentArea";

function App() {
  return (
    <MainLayout>
      <ContentArea>
        <div className="flex h-full items-center justify-center">
          <h1 className="text-2xl font-bold text-gray-500">
            Select a group to view combos
          </h1>
        </div>
      </ContentArea>
    </MainLayout>
  );
}

export default App;
