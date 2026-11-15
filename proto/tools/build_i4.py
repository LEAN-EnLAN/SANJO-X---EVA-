import csv, json, sys
crew = {}
def load(f, tcol, panel):
    for x in csv.DictReader(open(f)):
        a = x['ANALYTE'].replace(' (FEMALE)', '')
        try: v = float(x['VALUE'])
        except: continue
        s = x['SUBJECT_ID']; e = crew.setdefault(s, {'sex': x['SEX'], 'labs': {}})
        L = e['labs'].setdefault(a, {'unit': x['UNITS'], 'panel': panel, 'lo': None, 'hi': None, 'v': {}})
        for k, c in (('lo','RANGE_MIN'),('hi','RANGE_MAX')):
            try: L[k] = float(x[c])
            except: pass
        L['v'][x[tcol]] = v
load(sys.argv[1], 'timepoint', 'CMP'); load(sys.argv[2], 'TEST_DATE', 'CBC')
json.dump({'source': 'NASA OSDR OSD-575 (CMP) y OSD-569 (CBC), SpaceX Inspiration4', 'timepoints': ['L-92','L-44','L-3','R+1','R+45','R+82','R+194'], 'crew': crew}, open(sys.argv[3], 'w'), separators=(',',':'), ensure_ascii=False)
