#!/usr/bin/env python3
# -*- coding: utf-8 -*-

# imports
import os 
import pickle
import time
import lib.LinkedinAPI.settings as settings

class Error(Exception):
    pass

class LinkedinSessionExpired(Error):
    pass

class CookieRepository(object):

    @staticmethod
    def save(cookies, username):
        CookieRepository._ensure_cookies_dir()
        cookiejar_filepath = CookieRepository._get_cookies_filepath(username)
        with open(cookiejar_filepath, "wb") as f:
            pickle.dump(cookies, f)
