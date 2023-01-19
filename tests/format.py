def flatten_ordered_dict(data, delimiter=':', prefix=''):
    """"
    F'latten nested dict to a single level, preserving the key path as the new key.
    """
    result = OrderedDict()
    for k, v in data.items():
        if isinstance(v, dict):
            new_prefix = prefix + delimiter + k if prefix else k
            result.update(flatten_ordered_dict(v, delimiter, new_prefix))
        else:
            if prefix:
                key = prefix + delimiter + k
            else:
                key = k
            result[key] = v
    return result

