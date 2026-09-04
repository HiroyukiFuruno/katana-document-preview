#!/usr/bin/env bash
set -euo pipefail

# Ubuntuイメージ更新でrenderer用カラー絵文字入力が変動しないよう固定する。
emoji_package='fonts-noto-color-emoji'
emoji_version='2.047-0ubuntu0.24.04.1'
emoji_package_url="https://archive.ubuntu.com/ubuntu/pool/main/f/fonts-noto-color-emoji/${emoji_package}_${emoji_version}_all.deb"
emoji_package_sha256='b102b62ce6a7313315223cff5e052bdf8fc0ad162c9e961ce0dc2202bc139ce2'
emoji_font='/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf'
emoji_font_sha256='93cdc4ee9aa40e2afceecc63da0ca05ec7aab4bec991ece51a6b52389f48a477'

sudo apt-get update
sudo apt-get install -y fonts-noto-cjk graphviz imagemagick xvfb xclip

package_tmp_dir="$(mktemp -d)"
trap 'rm -rf "${package_tmp_dir}"' EXIT
package_path="${package_tmp_dir}/${emoji_package}_${emoji_version}_all.deb"
curl --fail --location --silent --show-error "${emoji_package_url}" --output "${package_path}"
test "$(sha256sum "${package_path}" | cut -d ' ' -f 1)" = "${emoji_package_sha256}"
sudo dpkg --install "${package_path}"
test "$(sha256sum "${emoji_font}" | cut -d ' ' -f 1)" = "${emoji_font_sha256}"
echo "KUC_PINNED_LINUX_EMOJI_SHA256=${emoji_font_sha256}" >> "${GITHUB_ENV:?GITHUB_ENV must be set by GitHub Actions}"

sudo mkdir -p /opt/local/bin
sudo ln -sf /usr/bin/dot /opt/local/bin/dot
echo 'GRAPHVIZ_DOT=/opt/local/bin/dot' >> "${GITHUB_ENV}"
