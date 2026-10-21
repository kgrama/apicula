"""Export apycula.attrids dict tables to data/attrids.json for the Rust gowin_unpack."""
import json, os
from apycula import attrids
out = {n: {str(k): v for k, v in getattr(attrids, n).items()}
       for n in dir(attrids) if not n.startswith('_') and isinstance(getattr(attrids, n), dict)}
path = os.path.join(os.path.dirname(__file__), 'data', 'attrids.json')
with open(path, 'w') as f:
    json.dump(out, f, indent=0)
print(path, len(out))
