import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { isTauri } from "@tauri-apps/api/core";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { DevBanner } from "@/components/DevBanner";
import { ServerStatus } from "@/components/ServerStatus";
import { RequestLog } from "@/components/RequestLog";
import { AllowlistEditor } from "@/components/AllowlistEditor";
import { RemoteDeckEditor } from "@/components/RemoteDeckEditor";
import { SettingsPanel } from "@/components/SettingsPanel";
import { ThemeToggle } from "@/components/ThemeToggle";
import { getServerStatus } from "@/lib/tauri";

function App() {
  const [activeTab, setActiveTab] = useState("allowlist");
  const [version, setVersion] = useState<string | null>(null);

  useEffect(() => {
    if (!isTauri()) {
      return;
    }

    void getServerStatus().then((status) => setVersion(status.version));
  }, []);

  useEffect(() => {
    if (!isTauri()) {
      return;
    }

    let unlisten: (() => void) | undefined;

    void listen<string>("navigate-tab", (event) => {
      if (event.payload) {
        setActiveTab(event.payload);
      }
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      unlisten?.();
    };
  }, []);

  return (
    <div className="traylink-theme traylink-app-shell min-h-screen text-foreground">
      <DevBanner />
      <header className="border-b border-border px-6 py-4">
        <div className="flex items-start justify-between gap-4">
          <div className="flex items-center gap-3">
            <img
              src="/icon.png"
              alt="TrayLink"
              className="h-10 w-10 rounded-lg object-cover"
            />
            <div>
              <p className="text-[11px] font-semibold uppercase tracking-[0.18em] text-muted-foreground">
                TrayLink
              </p>
              <h1 className="text-2xl font-semibold tracking-tight text-foreground">Dashboard</h1>
              <p className="text-sm text-muted-foreground">
                App launcher chạy nền — HTTP API trên localhost
              </p>
            </div>
          </div>
          <ThemeToggle />
        </div>
        <p className="mt-1 text-sm text-muted-foreground">
          {version ? (
            <>
              Version{" "}
              <span className="font-medium tabular-nums text-foreground">{version}</span>
              {" · "}
            </>
          ) : null}
          Source code:{" "}
          <a
            href="https://github.com/PhamMinhKha/TrayLink"
            target="_blank"
            rel="noreferrer"
            className="text-foreground underline underline-offset-4 hover:text-foreground/80"
          >
            github.com/PhamMinhKha/TrayLink
          </a>
        </p>
      </header>

      <main className="p-6">
        <Tabs value={activeTab} onValueChange={setActiveTab} className="space-y-4">
          <TabsList>
            <TabsTrigger value="allowlist">Apps & Commands</TabsTrigger>
            <TabsTrigger value="remote">Remote Deck</TabsTrigger>
            <TabsTrigger value="overview">Overview</TabsTrigger>
            <TabsTrigger value="logs">Request Log</TabsTrigger>
            <TabsTrigger value="settings">Settings</TabsTrigger>
          </TabsList>

          <TabsContent value="allowlist">
            <AllowlistEditor />
          </TabsContent>

          <TabsContent value="remote" forceMount className="data-[state=inactive]:hidden">
            <RemoteDeckEditor active={activeTab === "remote"} />
          </TabsContent>

          <TabsContent value="overview">
            <ServerStatus />
          </TabsContent>

          <TabsContent value="logs">
            <RequestLog />
          </TabsContent>

          <TabsContent value="settings">
            <SettingsPanel />
          </TabsContent>
        </Tabs>
      </main>
    </div>
  );
}

export default App;
