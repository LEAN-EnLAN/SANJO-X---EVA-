# AI-generated content notice

This notice covers the images in `landing/a/`. Team decision (Sun 15/11): the AI declaration lives here, in `landing/README.md` and in the project description, not as a mark on each image. The landing page keeps one short line in its footer that points here.

## AI-generated images
- **Codex images** (`ic-*`, `wrist-*`, `chip-*`, `cab-*`, `pe-*`, `pm-*`, `ps-*`, `ms1.jpg`, `s*`, `st*`, `sc*`, `fl*`, `pt1-*`): made with the built-in OpenAI ImageGen tool in Codex, from prompts by team Sanjo-X written with Claude. The tool did not report its exact image model. Some images used earlier AI-generated images as references.
- **Team reference images, v2** (`band2-joined.webp`, `band2-stacked.webp`, `band2-parts.webp`, `station-front.webp`, `station-wide.webp`, `station-doors-open.webp`): AI-generated concept images that the team shared on Sun 15/11 as the design references for the v2 wristband and the emergency station. They are interim visuals; Codex renders will replace them under the same file names. The station images carry Spanish labels, so the page overlays the English product text.

## Made without AI
- The "How it links" diagram, the suit-mode cross-section and the screen overlays on the page are inline SVG and HTML written by the team with Claude. They are not images.
- The 3D Sun texture (`sun-nasa.jpg`) is NASA's free model from NASA 3D Resources.

## Post-processing
Done locally with Python and Pillow: resizing, WebP conversion (v2 references at 1400 px, quality 82), edges blended into black or white, and contact shadows drawn from each layer's alpha. The small "AI-generated" mark that earlier versions stamped in the bottom-right corner was removed from the images the page still uses, by filling that corner from the surrounding pixels.

## Content rules
The images are illustrative concept art. They are not photographs or NASA imagery, they contain no NASA insignia or branding, and they depict no real people.
