### Exports utility functions when working with the project. Can also be put in shell config without the exports.

### FOR ZSH ONLY

### Copy the absolute path of this file to your .zshrc to get access to these sweet helpers. Basically they let you
### run diesel stuff without having to cd to the directory where diesel.toml is.

# Run from project root. Generates diesel migraion.
dmgen() {
  S=$(pwd)
  cd $(find . -name "diesel.toml" -type f -not -path "*/tests/*" -exec dirname {} \;) && diesel migration generate "$1" && cd $S
}

# Run from project root. Runs migrations on local postgres server on the provided DB.
dmrun() {
  # Save current to get back to after we're done
  S=$(pwd)

  # Check if the env variable arleady exists and prompt DB name if it doesn't
  if [ -z "${DATABASE_URL}" ]; then
    echo "Enter the database name: "
    read -r "DB_NAME"
    if [ -z "${DB_NAME}" ]; then
      echo "DB_NAME cannot be empty!"
      return
    fi
  fi

  # Export the DB URL
  export DATABASE_URL="postgres://postgres:postgres@localhost:5432/${DB_NAME}"

  echo "Searching for diesel.toml in $(pwd)"

  # Find the diesel.toml file and run the migrations in that dir
  cd $(find . -name "diesel.toml" -type f -not -path "*/tests/*" -exec dirname {} \;) && diesel migration run && cd $S
}

# Run from project root. Reverts migration on local postgres server on the provided DB.
dmrev() {
  S=$(pwd)

  if [ -z "${DATABASE_URL}" ]; then
    echo "Enter the database name: "
    read -r "DB_NAME"
    if [ -z "${DB_NAME}" ]; then
      echo "DB_NAME cannot be empty!"
      return
    fi
  fi

  export DATABASE_URL="postgres://postgres:postgres@localhost:5432/${DB_NAME}"

  echo "Searching for diesel.toml in $(pwd)"

  cd $(find . -name "diesel.toml" -type f -not -path "*/tests/*" -exec dirname {} \;) && diesel migration revert && cd $S
}
