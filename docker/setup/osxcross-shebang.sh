#!/bin/sh

# osxcross has some sh files without the shebang, causing "Exec format errors"
# when invoked by cc-rs
main() {
  for file in /opt/osxcross/target/bin/*; do
    if [ "$(file -b --mime-type "$file")" = "text/plain" ]; then
      printf "#!/bin/sh\n%s\n" "$(cat "$file")" > "$file"
    fi
  done

  rm "${0}"
}

main "${@}"
