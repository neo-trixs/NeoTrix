import os
import re

src_dir = "neotrix-core/src"
count = 0
total_replacements = 0

# Pattern to match .unwrap() as a method call:
# - preceded by ), ], or } (end of expression)
# - followed by ;, newline, whitespace, ., ,, or end of line
# This avoids matching .unwrap() inside string literals in most cases
UNWRAP_PATTERN = re.compile(r'([\)\]\}])\.unwrap\(\)')

for root, dirs, files in os.walk(src_dir):
    for f in files:
        if not f.endswith('.rs'):
            continue
        filepath = os.path.join(root, f)
        with open(filepath, 'r') as fh:
            content = fh.read()
        
        is_test_file = '/tests/' in filepath or f.startswith('test_')
        if not is_test_file:
            if '#[cfg(test)]' not in content:
                continue
        
        new_content, replacements = UNWRAP_PATTERN.subn(r'\1.expect("test")', content)
        if replacements > 0:
            with open(filepath, 'w') as fh:
                fh.write(new_content)
            count += 1
            total_replacements += replacements

print(f"Modified {count} files, {total_replacements} replacements")
