#!/bin/sh
echo "# TODO"
echo '
# FIXME
'
echo \# FIXME
echo example#FIXME
echo ${value#FIXME}
echo "$(echo "# TODO")"
echo $(echo sample)#FIXME
echo "$(case sample in sample) echo "# FIXME";; esac)"
cat <<-'TEXT'
	# FIXME
	TEXT
# TODO(fisher-a2q.1): Fill in the example.
