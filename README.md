# socialosint
A python osint tool for getting emails, from a target, published in social networks like Instagram, Linkedin and Twitter for finding the possible credential leaks in PwnDB

[![forthebadge](https://forthebadge.com/images/badges/made-with-python.svg)](https://forthebadge.com)

# Installation
```
git clone https://github.com/krishpranav/socialosint
cd socialosint
python3 -m pip install -r requirements.txt
python3 socialosint.py
```

# Usage

- you need to give your credentials here 

```
only for instagram & linkedin you need to give
```

```
{
    "instagram":{
        "username":"username",
        "password":"password"
    },
    "linkedin":{
        "email":"email",
        "password":"password"
    }
}
```

# Examples

# Instagram example:
```
python3 socialosint.py --credentials credentials.json --instagram --info somename
python3 socialosint.py --credentials credentials.json --instagram --location 832578276
python3 socialosint.py --credentials credentials.json --instagram --hashtag-ig someHashtag --pwndb
python3 socialosint.py --credentials credentials.json --instagram --target-ig username --pwndb
python3 socialosint.py --credentials credentials.json --instagram --target-ig username --followers-ig --followings-ig --pwndb
```

# Linkedin example:
```
python3 socialpwned.py --credentials credentials.json --linkedin --search-companies "My Target"
python3 socialpwned.py --credentials credentials.json --linkedin --search-companies "My Target" --employees --pwndb
python3 socialpwned.py --credentials credentials.json --linkedin --company 123456789 --employees --pwndb
python3 socialpwned.py --credentials credentials.json --linkedin --company 123456789 --employees --add-contacts
python3 socialpwned.py --credentials credentials.json --linkedin --user-contacts user-id --pwndb
python3 socialpwned.py --credentials credentials.json --linkedin --user-contacts user-id --add-contacts
```