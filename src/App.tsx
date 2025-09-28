/**
 * @file App.tsx
 *
 * Main application component.
 * Handles UI state, user interactions, and orchestrates communication
 * with the Rust backend via Tauri APIs.
 */

import { open, save } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { Button } from "./components/ui/button";
import { Checkbox } from "./components/ui/checkbox";
import { Input } from "./components/ui/input";
import { Label } from "./components/ui/label";
import { Progress } from "./components/ui/progress";
import { useState, useEffect, useRef, useCallback } from "preact/hooks";
import { Settings, MapPin, Gauge, FolderOpen } from "lucide-react";

function App() {
  // State hooks for managing UI and the conversion process.
  const [mapPath, setMapPath] = useState<string | null>(null);
  const [vmfPath, setVMFPath] = useState<string | null>(null);
  const [isConverting, setIsConverting] = useState(false);

  // 3-level progress system
  const [detailProgress, setDetailProgress] = useState(0);
  const [detailLabel, setDetailLabel] = useState("Ready");
  const [stageProgress, setStageProgress] = useState(0);
  const [stageLabel, setStageLabel] = useState("Ready");
  const [overallProgress, setOverallProgress] = useState(0);
  const [overallLabel, setOverallLabel] = useState("Ready");

  const [useOptimization, setUseOptimization] = useState(false);
  const [openOnComplete, setOpenOnComplete] = useState(true);

  // New state for dynamic coordinates and manual input
  const [useDynamicCoordinates, setUseDynamicCoordinates] = useState(true);
  const [startX, setStartX] = useState("");
  const [startZ, setStartZ] = useState("");
  const [endX, setEndX] = useState("");
  const [endZ, setEndZ] = useState("");
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);

  // Ref to hold the current value of `openOnComplete` to avoid stale closures in listeners.
  const openOnCompleteRef = useRef(openOnComplete);
  const settingsRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    openOnCompleteRef.current = openOnComplete;
  }, [openOnComplete]);

  // Close settings dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        settingsRef.current &&
        !settingsRef.current.contains(event.target as Node)
      ) {
        setIsSettingsOpen(false);
      }
    };

    if (isSettingsOpen) {
      document.addEventListener("mousedown", handleClickOutside);
    }

    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [isSettingsOpen]);

  // Memoized handler functions to ensure stable references.
  const selectMapDirectory = useCallback(async () => {
    const selected = await open({ multiple: false, directory: true });
    if (typeof selected === "string") {
      setMapPath(selected);
    }
  }, []);

  const selectVMFFile = useCallback(async () => {
    const selected = await save({
      filters: [{ name: "Valve Map File", extensions: ["vmf"] }],
    });
    if (selected) {
      setVMFPath(selected);
    }
  }, []);

  const handleConvert = useCallback(async () => {
    if (!mapPath || !vmfPath) {
      alert("Please select both the map folder and the output VMF file path.");
      return;
    }

    let coords = null;
    if (!useDynamicCoordinates) {
      const sx = parseInt(startX, 10);
      const sz = parseInt(startZ, 10);
      const ex = parseInt(endX, 10);
      const ez = parseInt(endZ, 10);

      if (isNaN(sx) || isNaN(sz) || isNaN(ex) || isNaN(ez)) {
        alert("Please enter valid integer coordinates for all fields.");
        return;
      }
      coords = { start_x: sx, start_z: sz, end_x: ex, end_z: ez };
    }

    setIsConverting(true);
    setDetailProgress(0);
    setStageProgress(0);
    setOverallProgress(0);
    setDetailLabel("Initializing...");
    setStageLabel("Starting conversion...");
    setOverallLabel("Beginning process...");

    try {
      await invoke("start_conversion", {
        mapPath: mapPath,
        vmfPath: vmfPath,
        optimize: useOptimization,
        dynamic: useDynamicCoordinates,
        coords: coords,
      });
    } catch (error) {
      console.error("Conversion failed:", error);
      alert(`An error occurred during conversion: ${error}`);
      setIsConverting(false);
      setDetailLabel("Error occurred");
      setStageLabel("Conversion failed");
      setOverallLabel("Failed");
    }
  }, [
    mapPath,
    vmfPath,
    useOptimization,
    useDynamicCoordinates,
    startX,
    startZ,
    endX,
    endZ,
  ]);

  // Effect to set up and tear down event listeners for backend communication.
  useEffect(() => {
    const unlistenFuncs: Array<() => void> = [];

    const setupListeners = async () => {
      // Detail progress (chunks, blocks, etc.)
      const unlistenDetail = await listen<{ progress: number; label: string }>(
        "detail_progress",
        (event) => {
          setDetailProgress(event.payload.progress);
          setDetailLabel(event.payload.label);
        }
      );

      // Stage progress (Reading, Converting, etc.)
      const unlistenStage = await listen<{ progress: number; label: string }>(
        "stage_progress",
        (event) => {
          setStageProgress(event.payload.progress);
          setStageLabel(event.payload.label);
        }
      );

      // Overall progress (0-100% of entire process)
      const unlistenOverall = await listen<{ progress: number; label: string }>(
        "overall_progress",
        (event) => {
          setOverallProgress(event.payload.progress);
          setOverallLabel(event.payload.label);
        }
      );

      const unlistenComplete = await listen<string>(
        "conversion_complete",
        (event) => {
          setIsConverting(false);
          setDetailProgress(100);
          setDetailLabel("Complete");
          setStageProgress(100);
          setStageLabel("File saved successfully");
          setOverallProgress(100);
          setOverallLabel("Conversion completed");

          if (openOnCompleteRef.current) {
            invoke("open_file_location", { path: event.payload }).catch((err) =>
              console.error("Could not open file location:", err)
            );
          }
        }
      );

      unlistenFuncs.push(
        unlistenDetail,
        unlistenStage,
        unlistenOverall,
        unlistenComplete
      );
    };

    setupListeners();

    return () => {
      unlistenFuncs.forEach((unlisten) => unlisten());
    };
  }, []);

  return (
    <main class="bg-background h-screen w-screen flex flex-col justify-around items-center p-4">
      <h1 className={"font-bold text-center text-xl"}>
        MTS - Minecraft to source
      </h1>

      {/* Path selection section */}
      <div className={"w-full grid grid-cols-6 items-center"}>
        <div className={"col-start-2 col-span-4 flex flex-col gap-4"}>
          <div className="flex flex-row gap-2">
            <Input
              value={mapPath || ""}
              placeholder={"Provide the path to the map folder..."}
              disabled={true}
            />
            <Button
              variant={"outline"}
              onClick={selectMapDirectory}
              disabled={isConverting}
            >
              Select Folder
            </Button>
          </div>
          <div className="flex flex-row gap-2">
            <Input
              value={vmfPath || ""}
              placeholder={"Provide a path to save the VMF file..."}
              disabled={true}
            />
            <Button
              variant={"outline"}
              onClick={selectVMFFile}
              disabled={isConverting}
            >
              Select file
            </Button>
          </div>
        </div>
      </div>

      {/* 3-Level Progress bars section */}
      <div className={"w-full flex flex-col gap-4 justify-center items-center"}>
        {/* Detail Progress (Top) */}
        <div className="w-full flex flex-col gap-2 justify-center items-center">
          <Label className="text-sm text-muted-foreground">{detailLabel}</Label>
          <Progress value={detailProgress} className={"w-2/5"} />
          <Label className="text-xs text-muted-foreground">
            {detailProgress}%
          </Label>
        </div>

        {/* Stage Progress (Middle) */}
        <div className="w-full flex flex-col gap-2 justify-center items-center">
          <Label className="text-base font-medium">{stageLabel}</Label>
          <Progress value={stageProgress} className={"w-2/5"} />
          <Label className="text-sm text-muted-foreground">
            {stageProgress}%
          </Label>
        </div>

        {/* Overall Progress (Bottom) */}
        <div className="w-full flex flex-col gap-2 justify-center items-center">
          <Label className="text-lg font-semibold">{overallLabel}</Label>
          <Progress value={overallProgress} className={"w-2/5 h-3"} />
          <Label className="text-base font-medium">{overallProgress}%</Label>
        </div>
      </div>

      {/* Manual Coordinates Input */}
      {!useDynamicCoordinates && (
        <div class="w-2/5 p-4 border rounded-md transition-all duration-300">
          <div className="grid grid-cols-2 gap-4">
            <Input
              type="number"
              placeholder="Start X"
              value={startX}
              onInput={(e) => setStartX((e.target as HTMLInputElement).value)}
              disabled={isConverting}
            />
            <Input
              type="number"
              placeholder="Start Z"
              value={startZ}
              onInput={(e) => setStartZ((e.target as HTMLInputElement).value)}
              disabled={isConverting}
            />
            <Input
              type="number"
              placeholder="End X"
              value={endX}
              onInput={(e) => setEndX((e.target as HTMLInputElement).value)}
              disabled={isConverting}
            />
            <Input
              type="number"
              placeholder="End Z"
              value={endZ}
              onInput={(e) => setEndZ((e.target as HTMLInputElement).value)}
              disabled={isConverting}
            />
          </div>
        </div>
      )}

      {/* Controls section with Settings Dropdown */}
      <div className={"flex flex-row gap-4 items-center"}>
        <div className="relative" ref={settingsRef}>
          <Button
            variant={"outline"}
            size={"icon"}
            disabled={isConverting}
            onClick={() => setIsSettingsOpen(!isSettingsOpen)}
          >
            <Settings className="h-4 w-4" />
          </Button>

          {isSettingsOpen && (
            <div className="absolute bottom-full mb-2 right-0 w-fit flex flex-col gap-4 bg-background border-border border-[1px] rounded-md shadow-lg z-50 px-2 py-4">
              <Label>Conversion Settings</Label>
              <div class={"flex flex-col gap-2"}>
                <Button
                  className="flex justify-between items-center py-6"
                  variant={"outline"}
                  onClick={() => {
                    if (!isConverting)
                      setUseDynamicCoordinates(!useDynamicCoordinates);
                  }}
                  disabled={isConverting}
                >
                  <MapPin className="h-4 w-4" />
                  <div className="flex flex-col items-start flex-1">
                    <span className="font-medium">Dynamic Coordinates</span>
                    <span className="text-xs text-muted-foreground">
                      Skip empty chunks automatically
                    </span>
                  </div>
                  <Checkbox
                    checked={useDynamicCoordinates}
                    readOnly
                    className="pointer-events-none"
                  />
                </Button>

                <Button
                  className="flex justify-between items-center py-6"
                  variant={"outline"}
                  onClick={() => {
                    if (!isConverting) setUseOptimization(!useOptimization);
                  }}
                  disabled={isConverting}
                >
                  <Gauge className="h-4 w-4" />
                  <div className="flex flex-col items-start flex-1">
                    <span className="font-medium">Optimization Mode</span>
                    <span className="text-xs text-muted-foreground">
                      Optimize VMF output
                    </span>
                  </div>
                  <Checkbox
                    checked={useOptimization}
                    readOnly
                    className="pointer-events-none"
                  />
                </Button>

                <Button
                  className="flex justify-between items-center py-6"
                  variant={"outline"}
                  onClick={() => {
                    if (!isConverting) setOpenOnComplete(!openOnComplete);
                  }}
                  disabled={isConverting}
                >
                  <FolderOpen className="h-4 w-4" />
                  <div className="flex flex-col items-start flex-1">
                    <span className="font-medium">Auto-open Location</span>
                    <span className="text-xs text-muted-foreground">
                      Open file location when done
                    </span>
                  </div>
                  <Checkbox
                    checked={openOnComplete}
                    readOnly
                    className="pointer-events-none"
                  />
                </Button>
              </div>
            </div>
          )}
        </div>

        <Button
          variant={"outline"}
          onClick={handleConvert}
          disabled={isConverting || !mapPath || !vmfPath}
        >
          {isConverting ? "Converting..." : "Convert"}
        </Button>
      </div>

      <Label className={"absolute bottom-2 right-4 text-sm text-muted"}>
        &copy; 2025 Drapco
      </Label>
    </main>
  );
}

export default App;
