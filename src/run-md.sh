MD_FILE_PATH=$1
MD_FILE_NAME=$(basename "$MD_FILE_PATH")
TAG=$2
mkdir -p .run-md/
awk -v tag="$TAG" '
  # Match lines like: [//]: # (TAG: description)
  $0 ~ ("^[[]//[]]: # \\(" tag ": ") {
      desc = $0
      # strip prefix: [//]: # (TAG:
      sub("^[[]//[]]: # \\(" tag ": ", "", desc)
      # strip trailing )
      sub("[)]$", "", desc)
      tagged=1
      next
  }

  /^```sh$/ && tagged {
      inblock=1
      tagged=0
      print "echo \"##### Executing Section #####\""
      print "echo " "\"    " desc "\""
      print "echo \"#############################\""
      next
  }

  /^```/ && inblock {
      inblock=0
      print "echo \"##### Executed Section #####\""
      print "echo " "\"    " desc "\""
      print "echo \"############################\""
      print ""
      next
  }
  inblock { print }
' "$MD_FILE_PATH" > ".run-md/$MD_FILE_NAME-$TAG.sh"
$SHELL ".run-md/$MD_FILE_NAME-$TAG.sh"
