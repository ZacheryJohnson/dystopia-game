# Given Names

```bash
curl https://raw.githubusercontent.com/hadley/data-baby-names/master/baby-names.csv | awk -F "," '{print $2}' | sort | uniq > given_names.txt
sed -i -E "s/\"(.*)\"/\1/" first_names.txt 
```

Needs manual cleaning of last line, which is `"name"`.

# Surnames

```bash
curl https://raw.githubusercontent.com/fivethirtyeight/data/master/most-common-name/surnames.csv | awk -F "," '{print $1}' | head -n7000 | sort | uniq > surnames.txt
sed -i -E "s/(\w)(\w*)/\u\1\L\2/" surnames.txt
```