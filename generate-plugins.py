import os
import toml

def generate_plugins_md():
    plugins_dir = "plugins"
    output_file = "plugins.md"
    plugin_dirs = [d for d in os.listdir(plugins_dir) if os.path.isdir(os.path.join(plugins_dir, d))]
    with open(output_file, "w") as f:
        f.write("# LLA Plugins\n\n")
        f.write("This document lists all available plugins for LLA and provides installation instructions.\n\n")
        f.write("## Installation\n\n")
        f.write("You can install all plugins at once using:\n\n")
        f.write("```bash\nlla install --git https://github.com/triyanox/lla\n```\n\n")
        f.write("Or you can install individual plugins as described below.\n\n")
        f.write("## Available Plugins\n\n")
        for plugin_dir in plugin_dirs:
            cargo_toml_path = os.path.join(plugins_dir, plugin_dir, "Cargo.toml")
            if os.path.exists(cargo_toml_path):
                with open(cargo_toml_path, "r") as toml_file:
                    cargo_data = toml.load(toml_file)
                
                name = cargo_data["package"]["name"]
                description = cargo_data["package"].get("description", "No description provided.")
                version = cargo_data["package"]["version"]

                f.write(f"### {name}\n\n")
                f.write(f"**Description:** {description}\n\n")
                f.write(f"**Version:** {version}\n\n")
                f.write("**Installation Options:**\n\n")
                
                f.write("1. Using LLA install command:\n")
                f.write("```bash\n")
                f.write(f"lla install --dir path/to/lla/plugins/{plugin_dir}\n")
                f.write("```\n\n")
                f.write("2. Manual installation:\n")
                f.write("```bash\n")
                f.write("git clone https://github.com/triyanox/lla\n")
                f.write(f"cd lla/plugins/{plugin_dir}\n")
                f.write("cargo build --release\n")
                f.write("```\n\n")
                f.write(f"Then, copy the generated `.so`, `.dll`, or `.dylib` file from the `target/release` directory to your LLA plugins directory.\n\n")
    print(f"Generated {output_file}")

if __name__ == "__main__":
    generate_plugins_md()