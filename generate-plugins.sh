#!/bin/bash

# Output file
output_file="plugins.md"

# Write header
cat > "$output_file" << EOL
# LLA Plugins

This document lists all available plugins for LLA and provides installation instructions.

## Installation

You can install all plugins at once using:

\`\`\`bash
lla install --git https://github.com/triyanox/lla
\`\`\`

Or you can install individual plugins as described below.

## Available Plugins

EOL

# Iterate through plugin directories
for plugin_dir in plugins/*/; do
    if [ -f "${plugin_dir}Cargo.toml" ]; then
        # Extract information from Cargo.toml using grep and sed
        name=$(grep '^name' "${plugin_dir}Cargo.toml" | sed 's/name = "\(.*\)"/\1/')
        version=$(grep '^version' "${plugin_dir}Cargo.toml" | sed 's/version = "\(.*\)"/\1/')
        description=$(grep '^description' "${plugin_dir}Cargo.toml" | sed 's/description = "\(.*\)"/\1/')
        
        # If description is empty, provide default
        if [ -z "$description" ]; then
            description="No description provided."
        fi
        
        # Write plugin information
        cat >> "$output_file" << EOL
### ${name}

**Description:** ${description}

**Version:** ${version}

**Installation Options:**

1. Using LLA install command:
\`\`\`bash
lla install --dir path/to/lla/${plugin_dir}
\`\`\`

2. Manual installation:
\`\`\`bash
git clone https://github.com/triyanox/lla
cd lla/${plugin_dir}
cargo build --release
\`\`\`

Then, copy the generated \`.so\`, \`.dll\`, or \`.dylib\` file from the \`target/release\` directory to your LLA plugins directory.

EOL
    fi
done

echo "Generated $output_file" 