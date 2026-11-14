"""Build a standalone news feature ZIP and its catalog entry."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import zipfile
from package_support import notices

ROOT=Path(__file__).resolve().parents[1]


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--profile',choices=('debug','release'),default='release')
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--base-url',default='https://example.invalid/news-service')
    args=parser.parse_args()
    output=args.output.resolve()
    if not output.is_relative_to(ROOT/'target') or output.exists():raise ValueError('fresh output inside target required')
    destination=output/'news';(destination/'bin').mkdir(parents=True)
    manifest=json.loads((ROOT/'module.json').read_text(encoding='utf-8'))
    shutil.copyfile(ROOT/'target'/args.profile/'lazybuddy-news.exe',destination/'bin/lazybuddy-news.exe')
    for name in ('module.json','README.md','NewsNow-LICENSE.txt'):shutil.copyfile(ROOT/name,destination/name)
    name='LICENSE' if (ROOT/'LICENSE').is_file() else 'NOTICE.txt'
    shutil.copyfile(ROOT/name,destination/name)
    notices(destination/'licenses',ROOT/'Cargo.toml',('lazybuddy-news',))
    archive=output/f'news-{manifest["version"]}-windows-x86_64.zip'
    with zipfile.ZipFile(archive,'w',compression=zipfile.ZIP_DEFLATED,compresslevel=6) as package:
        for path in sorted(destination.rglob('*')):
            if path.is_file():package.write(path,path.relative_to(destination).as_posix())
    with archive.open('rb') as source:digest=hashlib.file_digest(source,'sha256').hexdigest()
    entry=dict(id=manifest['id'],name=manifest['name'],kind=manifest.get('kind','feature'),version=manifest['version'],target=manifest['target'],dependencies=manifest['dependencies'],url=args.base_url.rstrip('/')+'/'+archive.name,sha256=digest,bytes=archive.stat().st_size)
    (output/'catalog.json').write_text(json.dumps(dict(schema=1,packages=[entry]),indent=2),encoding='utf-8')
    print(json.dumps(dict(archive=str(archive),bytes=entry['bytes'])))


if __name__=='__main__':main()
