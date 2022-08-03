from itertools import combinations, zip_longest, chain
import json

from get_available_features import get_available_features

def grouper(iterable, n, *, incomplete='fill', fillvalue=None):
    "Collect data into non-overlapping fixed-length chunks or blocks"
    # grouper('ABCDEFG', 3, fillvalue='x') --> ABC DEF Gxx
    # grouper('ABCDEFG', 3, incomplete='strict') --> ABC DEF ValueError
    # grouper('ABCDEFG', 3, incomplete='ignore') --> ABC DEF
    args = [iter(iterable)] * n
    if incomplete == 'fill':
        return zip_longest(*args, fillvalue=fillvalue)
    if incomplete == 'strict':
        return zip(*args, strict=True)
    if incomplete == 'ignore':
        return zip(*args)
    else:
        raise ValueError('Expected fill, strict, or ignore')

current_available_features = get_available_features()

features_chunks = map(  # split every features chunks with ' '
    lambda chunks: ' '.join(filter(lambda v: v is not None, chunks)),
    grouper(  # group into chunks with 15 features
        map(
            lambda features: ','.join(features),
            chain.from_iterable(  # flat
                map(  # calculate combinations
                    lambda i: combinations(current_available_features, i),
                    range(1, len(current_available_features) + 1)
                )
            ),
        ),
        15
    )
)

print(json.dumps(list(features_chunks)))
