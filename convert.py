#!/usr/bin/env python3
import json
def main():
    json_ = dict(defaultAction='SCMP_ACT_ERRNO',
                architectures=[
                    'SCMP_ARCH_X86_64'
                ])
    with open('whitelist', 'r') as fhandle:
        syscalls = [dict(name=i.replace('\n',''),action='SCMP_ACT_ALLOW',args=[])
                for i in fhandle]
    json_['syscalls'] = syscalls
    with open('whitelist.json', 'w') as fhandle:
       json.dump(json_, fhandle, indent=2, sort_keys=True)

if __name__ == '__main__':
    main()
