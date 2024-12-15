wrk.method = "POST"
wrk.headers["Content-Type"] = "application/json"
wrk.headers["CD-ID"] = "5c133a93-8dd4-4958-847a-ae81a5e11743"
wrk.headers["CD-Secret"] = "2fb5be09-8dba-481c-aaaf-5efad1d0a59c"
wrk.headers["Application-ID"] = "673d6733caa30090be5b410d"

wrk.body = [[{
  "error": "Sample error message",
  "traceback": "Sample traceback details with more\nlines of code",
  "url": "http://example.com",
  "method": "GET"
}]]

