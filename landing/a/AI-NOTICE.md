# AI-generated content notice

This notice covers the images in `landing/a/`. Team decision (Sun 15/11): the AI declaration lives here, in `landing/README.md` and in the project description, not as a mark on each image. The landing page keeps one short line in its footer that points here.

## AI-generated images
- **Codex images** (`ic-*`, `wrist-*`, `chip-*`, `cab-*`, `pe-*`, `pm-*`, `ps-*`, `ms1.jpg`, `s*`, `st*`, `sc*`, `fl*`, `pt1-*`): made with the built-in OpenAI ImageGen tool in Codex, from prompts by team Sanjo-X written with Claude. The tool did not report its exact image model. Some images used earlier AI-generated images as references.
- **Wristband v2 renders** (`band2-hero-white.webp`, `band2-exploded.webp`, `band2-base-alone.webp`, `band2-on-wrist.webp`, `band2-cabin.webp`, `band2-turntable-000.webp` to `band2-turntable-315.webp`): made with the built-in OpenAI ImageGen tool in Codex, from prompts by team Sanjo-X written with Claude. Used as delivered (WebP), with no post-processing.
- **Team reference images, v2** (`station-front.webp`, `station-wide.webp`, `station-doors-open.webp`): AI-generated concept images that the team shared on Sun 15/11 as the design references for the emergency station. They are interim visuals; Codex renders will replace them. The station images carry Spanish labels, which the page's English overlay names in English and the Spanish version repeats.

## Made without AI
- The "How it links" diagram, the suit-mode cross-section and the screen overlays on the page are inline SVG and HTML written by the team with Claude. They are not images.
- The 3D Sun texture (`sun-nasa.jpg`) is NASA's free model from NASA 3D Resources.

## Post-processing
Done locally with Python and Pillow: resizing, WebP conversion (v2 references at 1400 px, quality 82), edges blended into black or white, and contact shadows drawn from each layer's alpha. The small "AI-generated" mark that earlier versions stamped in the bottom-right corner was removed from the images the page still uses, by filling that corner from the surrounding pixels.

## Content rules
The images are illustrative concept art. They are not photographs or NASA imagery, they contain no NASA insignia or branding, and they depict no real people.
