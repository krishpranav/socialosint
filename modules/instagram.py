#!/usr/bin/env python3
# -*- coding: utf-8 -*-

# imports
import sys, requests, argparse, json, re, os, time
from modules.colors import colors
from lib.InstagramAPI import InstagramAPI
from lib.PwnDB import PwnDB

def getEmailsFromListOfUsers(api, items):
    targets = []
    print(colors.info + "Searching users..... \n" + colors.end)

    for item in items:
        user = str(item.get("user").get("username"))
        targets.append(getUserProfile(api, user))
    
    return getEmailsFromUsers(targets)

