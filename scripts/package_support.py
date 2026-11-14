import json,shutil,subprocess
from pathlib import Path

def notices(destination, manifest, names):
    metadata = json.loads(subprocess.check_output(
        ['cargo', 'metadata', '--locked', '--offline', '--format-version', '1', '--filter-platform', 'x86_64-pc-windows-msvc', '--manifest-path', str(manifest)], cwd=manifest.parent))
    packages = {package['id']: package for package in metadata['packages']}
    nodes = {node['id']: node for node in metadata['resolve']['nodes']}
    pending = [key for key, package in packages.items() if package['name'] in names]
    included = set()
    while pending:
        identifier = pending.pop()
        if identifier in included:
            continue
        included.add(identifier)
        pending.extend(nodes[identifier]['dependencies'])
    destination.mkdir(parents=True,exist_ok=True)
    records = []
    for identifier in sorted(included):
        package = packages[identifier]
        source = Path(package['manifest_path']).parent
        files = [path for path in source.iterdir() if path.is_file() and path.name.upper().startswith(('LICENSE', 'COPYING', 'NOTICE', 'COPYRIGHT'))]
        if files:
            directory = destination / f'{package["name"]}-{package["version"]}'
            directory.mkdir(parents=True)
            for path in files:
                shutil.copyfile(path, directory / path.name)
        records.append(dict(name=package['name'], version=package['version'], license=package['license'], repository=package['repository']))
    (destination / 'dependencies.json').write_text(json.dumps(records, ensure_ascii=False, indent=2), encoding='utf-8')

