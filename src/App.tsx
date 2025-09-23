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

function App() {
  // State hooks for managing UI and the conversion process.
  const [mapPath, setMapPath] = useState<string | null>(null);
  const [vmfPath, setVMFPath] = useState<string | null>(null);
  const [isConverting, setIsConverting] = useState(false);
  const [readingProgress, setReadingProgress] = useState(0);
  const [creatingProgress, setCreatingProgress] = useState(0);
  const [optimizationProgress, setOptimizationProgress] = useState(0); // State for the new progress bar
  const [useOptimization, setUseOptimization] = useState(false);
  const [openOnComplete, setOpenOnComplete] = useState(true);

  // Ref to hold the current value of `openOnComplete` to avoid stale closures in listeners.
  const openOnCompleteRef = useRef(openOnComplete);
  useEffect(() => {
    openOnCompleteRef.current = openOnComplete;
  }, [openOnComplete]);

  // Memoized handler functions to ensure stable references.
  const selectMapDirectory = useCallback(async () => {
    const selected = await open({ multiple: false, directory: true });
    if (typeof selected === 'string') {
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

    setIsConverting(true);
    setReadingProgress(0);
    setCreatingProgress(0);
    setOptimizationProgress(0); // Reset optimization progress on start.

    try {
      await invoke("start_conversion", {
        mapPath: mapPath,
        vmfPath: vmfPath,
        optimize: useOptimization,
      });
    } catch (error) {
      console.error("Conversion failed:", error);
      alert(`An error occurred during conversion: ${error}`);
      setIsConverting(false);
    }
  }, [mapPath, vmfPath, useOptimization]);

  // Effect to set up and tear down event listeners for backend communication.
  useEffect(() => {
    const unlistenFuncs: Array<() => void> = [];

    const setupListeners = async () => {
      const unlistenReading = await listen<number>("reading_progress", (event) => {
        setReadingProgress(event.payload);
      });

      // Listener for the new optimization progress event.
      const unlistenOptimizing = await listen<number>("optimization_progress", (event) => {
        setOptimizationProgress(event.payload);
      });
      
      const unlistenCreating = await listen<number>("creating_progress", (event) => {
        setCreatingProgress(event.payload);
      });

      const unlistenComplete = await listen<string>("conversion_complete", (event) => {
        setIsConverting(false);
        setReadingProgress(100);
        setCreatingProgress(100);
        setOptimizationProgress(100); // Ensure optimization bar is also full if it was used.

        if (openOnCompleteRef.current) {
          invoke("open_file_location", { path: event.payload })
            .catch(err => console.error("Could not open file location:", err));
        }
      });

      unlistenFuncs.push(unlistenReading, unlistenOptimizing, unlistenCreating, unlistenComplete);
    };

    setupListeners();

    return () => {
      unlistenFuncs.forEach(unlisten => unlisten());
    };
  }, []); // Empty dependency array ensures listeners are set up only once.


  return (
    <main class="bg-background h-screen w-screen flex flex-col justify-around items-center p-4">
      <h1 className={"font-bold text-center text-xl"}>MTS - Minecraft to source</h1>

      {/* Path selection section */}
      <div className={"w-full grid grid-cols-6 items-center"}>
        <div className={"col-start-2 col-span-4 flex flex-col gap-4"}>
          <div className="flex flex-row gap-2">
            <Input
              value={mapPath || ""}
              placeholder={"Provide the path to the map folder..."}
              disabled={true}
            />
            <Button variant={"outline"} onClick={selectMapDirectory} disabled={isConverting}>
              Select Folder
            </Button>
          </div>
          <div className="flex flex-row gap-2">
            <Input
              value={vmfPath || ""}
              placeholder={"Provide a path to save the VMF file..."}
              disabled={true}
            />
            <Button variant={"outline"} onClick={selectVMFFile} disabled={isConverting}>
              Select file
            </Button>
          </div>
        </div>
      </div>

      {/* Progress bars section */}
      <div className={"w-full flex flex-col gap-4 justify-center items-center"}>
        <div className="w-full flex flex-col gap-2 justify-center items-center">
          <Label>Reading map data: {readingProgress}%</Label>
          <Progress value={readingProgress} className={"w-2/5"} />
        </div>
        
        {/* Conditionally rendered optimization progress bar */}
        {useOptimization && (
          <div className="w-full flex flex-col gap-2 justify-center items-center transition-opacity duration-300">
            <Label>Optimizing data: {optimizationProgress}%</Label>
            <Progress value={optimizationProgress} className={"w-2/5"} />
          </div>
        )}
        
        <div className="w-full flex flex-col gap-2 justify-center items-center">
          <Label>Creating VMF file: {creatingProgress}%</Label>
          <Progress value={creatingProgress} className={"w-2/5"} />
        </div>
      </div>

      {/* Controls section */}
      <div className={"flex flex-col gap-8 items-center"}>
        <div className="flex flex-col gap-3">
          <div className={"flex flex-row gap-2 items-center"}>
            <Checkbox 
              id="optimization"
              checked={useOptimization}
              onCheckedChange={(checked) => setUseOptimization(Boolean(checked))}
              disabled={isConverting}
            />
            <Label htmlFor="optimization">Use optimization mode</Label>
          </div>
          <div className={"flex flex-row gap-2 items-center"}>
            <Checkbox 
              id="openOnComplete"
              checked={openOnComplete}
              onCheckedChange={(checked) => setOpenOnComplete(Boolean(checked))}
              disabled={isConverting}
            />
            <Label htmlFor="openOnComplete">Open file location on completion</Label>
          </div>
        </div>
        <Button variant={"outline"} onClick={handleConvert} disabled={isConverting}>
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