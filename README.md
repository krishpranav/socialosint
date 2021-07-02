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
python3 socialosint.py --credentials credentials.json --instagram --info España
python3 socialosint.py --credentials credentials.json --instagram --location 832578276
python3 socialosint.py --credentials credentials.json --instagram --hashtag-ig someHashtag --pwndb
python3 socialosint.py --credentials credentials.json --instagram --target-ig username --pwndb
python3 socialosint.py --credentials credentials.json --instagram --target-ig username --followers-ig --followings-ig --pwndb
```