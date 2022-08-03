'''
Deps
====

* cargo
* jq
'''

import json
import subprocess
from typing import List


def get_available_features() -> List[str]:
    subcmd = subprocess.run(
        'cargo read-manifest | jq ".features | keys"',
        shell=True,
        capture_output=True
    )
    outputs = subcmd.stdout.decode('utf-8')
    
    return json.loads(outputs)
