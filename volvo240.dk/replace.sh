for var in "$@"
do
    echo ===
    iconv  -t utf-8 -o $var $var
done
