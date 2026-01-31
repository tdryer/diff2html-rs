# Show recipes
default:
    just --list

image_name := "diff2html-devel"

# Run docker container
docker:
    podman build -t {{ image_name }} .
    podman run -it --rm --hostname devel --workdir /devel \
        --mount type=bind,src=${PWD},target=/devel \
        --mount type=volume,src=diff2html-target,target=/devel/target \
        --mount type=bind,src=${HOME}/.rustup,target=/root/.rustup \
        --mount type=bind,src=$(which kitten),target=/usr/local/bin/kitten,ro \
        --mount type=bind,src=$(which copilot),target=/usr/local/bin/copilot,ro \
        --mount type=bind,src=$(which just),target=/usr/local/bin/just,ro \
        --mount type=bind,src=${HOME}/.copilot/config.json,target=/root/.copilot/config.json \
        --mount type=bind,src=$(which claude),target=/root/.local/bin/claude \
        --mount type=bind,src=${HOME}/.claude,target=/root/.claude \
        --mount type=bind,src=${HOME}/.claude.json,target=/root/.claude.json \
        --mount type=bind,src=${HOME}/.gitconfig,target=/root/.gitconfig,ro \
        --mount type=bind,src=${HOME}/.gitconfig.private,target=/root/.gitconfig.private,ro \
        --env GITHUB_TOKEN=$(secret-tool lookup service copilot-cli) \
        --env UV_LINK_MODE=copy \
        --env TERM=${TERM} \
        --env IS_SANDBOX=1 \
        --env TZ=America/Vancouver \
        {{ image_name }} kitten run-shell --shell=bash
