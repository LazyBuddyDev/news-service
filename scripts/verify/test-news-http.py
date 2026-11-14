"""Local HTTP contract tests; never fetches upstream news."""
import json
import os
from pathlib import Path
import secrets
import subprocess
import unittest
import urllib.error
import urllib.request

ROOT = Path(__file__).resolve().parents[2]


class HttpTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.token = secrets.token_hex(24)
        cls.process = subprocess.Popen([str(ROOT / 'target/debug/lazybuddy-news-http.exe')],
            env=dict(os.environ, LAZYBUDDY_API_TOKEN=cls.token), stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL, text=True, encoding='utf-8', creationflags=subprocess.CREATE_NO_WINDOW)
        cls.address = json.loads(cls.process.stdout.readline())['address']

    @classmethod
    def tearDownClass(cls):
        cls.process.terminate()
        cls.process.wait(timeout=5)
        cls.process.stdout.close()

    def request(self, headers, body=None):
        request = urllib.request.Request(self.address, data=body or b'{"version":1,"id":"health","method":"health"}', headers=headers)
        try:
            return urllib.request.urlopen(request, timeout=5)
        except urllib.error.HTTPError as error:
            return error

    def test_auth_and_browser_origin_are_enforced(self):
        for headers in [{}, {'Authorization': 'Bearer incorrect'},
                        {'Authorization': 'Bearer ' + self.token, 'Origin': 'https://example.org'}]:
            with self.request(headers) as response:
                self.assertEqual(response.code, 401)

    def test_valid_client_and_body_limit(self):
        headers = {'Authorization': 'Bearer ' + self.token}
        with self.request(headers) as response:
            reply = json.load(response)
            self.assertEqual(reply['data']['service'], 'news')
        with self.request(headers, b' ' * 65537) as response:
            self.assertEqual(response.code, 413)


if __name__ == '__main__':
    unittest.main()
