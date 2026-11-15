// Prints the deck geometry from src/screens/dashboard/deck-data.js as JSON for tools/deck-base.blender.py.
import * as deck from "../src/screens/dashboard/deck-data.js"
process.stdout.write(JSON.stringify(deck))
