.PHONY: release
.SHELL: /bin/sh
.ONESHELL:

release:
	@echo -n "Enter version: "
	read version
	echo "Updating readme"
	cargo readme > README.md
	if git status -s | grep -q README.md; then
		git add README.md
		git commit -S -m "Updating README"
	fi
	echo "Updating Cargo.toml"
	sed -i "s/^version = .*/version = \"$$version\"/" Cargo.toml
	if git status -s | grep -q Cargo.toml; then
		git add Cargo.toml
		git commit -S -m "Updating Cargo.toml for $$version"
	fi
	git tag -s -m "$$version" $$version
	# cargo publish
	# git push
	# git push --tags
