with open('shaders.txt') as f:
    items = {}
    lines = filter(lambda s: s.startswith('uniform'), f.readlines())
    for line in lines:
        err = False
        try:
            detail = int(' '.join(line.split()[3:]))
            err = True
        except:
            pass
        type = line.split()[1].rstrip('>').lstrip('<')
        detail = ' '.join(line.split()[3:]) if not err else ''
        label = line.split()[2].rstrip(';')
        if label in items:
            continue
        items[label] = True
        detail = ' ' + detail if not detail == '' else ''
        print('{\n\tlabel: \'%s\',\n\tdetail: \'<%s>%s\'\n},' % (label, type, detail))