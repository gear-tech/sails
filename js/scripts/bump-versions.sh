version=$1

echo "Bumping versions to $version"

pkgs=("package.json" "./js/package.json" "./js/cli/package.json" "./js/parser/package.json" "./js/types/package.json" "./js/util/package.json")

for pkg in ${pkgs[@]}; do
  jq ".version = \"$version\"" $pkg > tmp.$$.json && mv tmp.$$.json $pkg
done
