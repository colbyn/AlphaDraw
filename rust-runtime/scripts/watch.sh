set -e

watchexec --exts rs --ignore 'target pkg' -- "./scripts/build.sh && echo '\n==reloaded==\n'"

