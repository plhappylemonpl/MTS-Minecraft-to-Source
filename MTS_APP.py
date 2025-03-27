# MTS_APP.py
import tkinter as tk
from tkinter import ttk, filedialog, messagebox
import logging
from MTS_world import get_and_convert_blocks

logging.basicConfig(level=logging.INFO)
log = logging.getLogger(__name__)

class BlockTextureUI:
    def __init__(self):
        self.window = tk.Tk()
        self.window.title("Minecraft to Source Converter")
        self.window.geometry("800x335")
        # Default paths
        self.world_path = tk.StringVar(value=r"C:\Users\lemon\AppData\Roaming\ModrinthApp\profiles\1.21.4\saves")
        self.output_path = tk.StringVar(value=r"N:\Gry\Steam\steamapps\common\GarrysMod\garrysmod\maps\test_mc")
        self.x1 = tk.StringVar(value="-65")
        self.z1 = tk.StringVar(value="-65")
        self.x2 = tk.StringVar(value="65")
        self.z2 = tk.StringVar(value="65")
        self.setup_ui()

    def setup_ui(self):
        # Paths
        path_frame = ttk.LabelFrame(self.window, text="Paths", padding="5")
        path_frame.pack(fill="x", padx=5, pady=5)
        ttk.Label(path_frame, text="Minecraft World:").grid(row=0, column=0, sticky="w")
        ttk.Entry(path_frame, textvariable=self.world_path, width=50).grid(row=0, column=1, padx=5)
        ttk.Button(path_frame, text="Browse", command=self.browse_world).grid(row=0, column=2)
        ttk.Label(path_frame, text="VMF Output:").grid(row=1, column=0, sticky="w")
        ttk.Entry(path_frame, textvariable=self.output_path, width=50).grid(row=1, column=1, padx=5)
        ttk.Button(path_frame, text="Browse", command=self.browse_output).grid(row=1, column=2)
        # Coordinates
        coord_frame = ttk.LabelFrame(self.window, text="Coordinates", padding="5")
        coord_frame.pack(fill="x", padx=5, pady=5)
        ttk.Label(coord_frame, text="X1:").grid(row=0, column=0)
        ttk.Entry(coord_frame, textvariable=self.x1, width=10).grid(row=0, column=1)
        ttk.Label(coord_frame, text="Z1:").grid(row=0, column=2)
        ttk.Entry(coord_frame, textvariable=self.z1, width=10).grid(row=0, column=3)
        ttk.Label(coord_frame, text="X2:").grid(row=1, column=0)
        ttk.Entry(coord_frame, textvariable=self.x2, width=10).grid(row=1, column=1)
        ttk.Label(coord_frame, text="Z2:").grid(row=1, column=2)
        ttk.Entry(coord_frame, textvariable=self.z2, width=10).grid(row=1, column=3)
        # Mirror settings
        self.mirror_axis = tk.StringVar(value="x")
        mirror_frame = ttk.LabelFrame(self.window, text="Mirror Settings", padding="5")
        mirror_frame.pack(fill="x", padx=5, pady=5)
        ttk.Label(mirror_frame, text="Mirror Axis (x/y):").grid(row=0, column=0, sticky="w")
        ttk.Combobox(mirror_frame, textvariable=self.mirror_axis, values=["x", "y"]).grid(row=0, column=1, padx=5)
        # Checkbox for optimization
        self.optimize_var = tk.BooleanVar(value=False)
        optimize_checkbox = ttk.Checkbutton(self.window, text="Optimize the grid (merge blocks) WIP, I do not recommend using it", variable=self.optimize_var)
        optimize_checkbox.pack(pady=5)
        # Convert button
        ttk.Button(self.window, text="Convert", command=self.convert).pack(pady=10)
        # Links to authors
        credit_frame = tk.Frame(self.window)
        credit_frame.pack(side="bottom", pady=5)
        credit_label = tk.Label(credit_frame, text="Original made by ", fg="black")
        credit_label.pack(side="left")
        daxen_link = tk.Label(credit_frame, text="Daxen", fg="blue", cursor="hand2")
        daxen_link.pack(side="left")
        daxen_link.bind("<Button-1>", lambda e: self.open_link("https://www.youtube.com/@iDaxen"))
        credit_label2 = tk.Label(credit_frame, text=" |   Fixed, added more functionality, improved responsiveness etc by ", fg="black")
        credit_label2.pack(side="left")
        lemon_link = tk.Label(credit_frame, text="LemoN", fg="goldenrod3", cursor="hand2")
        lemon_link.pack(side="left")
        lemon_link.bind("<Button-1>", lambda e: self.open_link("https://www.twitch.tv/lemonstreamuje"))

    def open_link(self, url):
        import webbrowser
        webbrowser.open_new(url)

    def run(self):
        self.window.mainloop()

    def browse_world(self):
        from tkinter import filedialog
        path = filedialog.askdirectory(title="Select Minecraft World Folder")
        if path:
            self.world_path.set(path)

    def browse_output(self):
        from tkinter import filedialog
        path = filedialog.asksaveasfilename(
            defaultextension=".vmf",
            filetypes=[("Valve Map File", "*.vmf")],
            title="Save VMF File As"
        )
        if path:
            self.output_path.set(path)

    def convert(self):
        try:
            get_and_convert_blocks(
                self.world_path.get(),
                int(self.x1.get()), int(self.z1.get()),
                int(self.x2.get()), int(self.z2.get()),
                self.output_path.get(),
                mirror_axis=self.mirror_axis.get(),
                optimize=self.optimize_var.get() 
            )
            messagebox.showinfo("Success", "Conversion completed successfully!")
        except Exception as e:
            log.exception("Conversion failed:")
            messagebox.showerror("Error", f"Conversion failed: {str(e)}")

if __name__ == "__main__":
    print("Starting UI...")
    app = BlockTextureUI()
    app.run()
