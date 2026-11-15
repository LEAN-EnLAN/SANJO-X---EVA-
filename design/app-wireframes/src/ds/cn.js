import { clsx } from "clsx"
import { extendTailwindMerge } from "tailwind-merge"
// teach tailwind-merge our custom font sizes so `text-display` is not mistaken for a colour
const twMerge = extendTailwindMerge({ extend: { classGroups: { "font-size": [{ text: ["display", "hero"] }] } } })
export const cn = (...a) => twMerge(clsx(a))
