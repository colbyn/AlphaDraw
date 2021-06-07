set -e

rsync -avz ./css dist
npx webpack && http-server dist