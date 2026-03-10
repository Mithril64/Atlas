# Atlas Documentation Index

Complete guide to all Atlas documentation and getting started.

---

## Quick Start (5 Minutes)

**New to Atlas?** Start here:

```bash
cd /home/mithril/code/Atlas
./quickstart.sh full
```

Then open:
- **Graph**: http://localhost:3000/../public/index.html
- **Submit**: http://localhost:3000/../public/submit.html

---

## Documentation by Role

### For Mathematicians & Researchers

**I want to submit my theorem/proof to the graph**

1. Read: **[CONTRIBUTOR_GUIDE.md](CONTRIBUTOR_GUIDE.md)**
   - How to format your submission
   - Required metadata
   - Common mistakes and fixes
   - Complete examples

2. Go to: **http://atlas.timgerasimov.com/submit.html**
   - Upload your Typst file
   - Get instant validation feedback
   - See your work in the graph

---

### For Backend Developers

**I want to understand the system architecture and extend it**

1. Read: **[ARCHITECTURE.md](ARCHITECTURE.md)**
   - System design philosophy
   - Data flow
   - How parsing works
   - Extensibility points

2. Read: **[API_REFERENCE.md](API_REFERENCE.md)**
   - All REST endpoints
   - Request/response formats
   - Error handling
   - Code examples in JS, Python, cURL

3. Review: `compiler/src/main.rs`
   - Clean, well-commented code
   - Three main functions: `ingest_submission()`, `validate_submission()`, `upload_handler()`

---

### For DevOps & Administrators

**I want to deploy Atlas to production and manage it**

1. Read: **[DEPLOYMENT.md](DEPLOYMENT.md)**
   - VPS setup with Systemd
   - Docker containerization
   - Nginx reverse proxy with SSL
   - Security hardening
   - Monitoring and logging

2. Use: **[quickstart.sh](quickstart.sh)**
   - Automated local testing
   - Validation of setup
   - Demo submission

3. Monitor: Check `math/submissions/` and `git log`
   - All user contributions are version-controlled
   - Easy to revert bad submissions
   - Clear audit trail

---

### For System Architects

**I want to understand the complete system design**

1. Read: **[SYSTEM_DIAGRAMS.md](SYSTEM_DIAGRAMS.md)**
   - High-level architecture
   - Data flow diagrams
   - Request/response flows
   - Deployment topology
   - State machines

2. Read: **[INTEGRATION_CHECKLIST.md](INTEGRATION_CHECKLIST.md)**
   - All implemented features
   - Validation pipeline details
   - Permissions and security
   - Testing procedures

3. Review: **[IMPLEMENTATION_SUMMARY.md](IMPLEMENTATION_SUMMARY.md)**
   - What was built
   - Technical decisions
   - Performance characteristics

---

## All Documentation

| Document | Audience | Length | Purpose |
|----------|----------|--------|---------|
| **[README.md](README.md)** | Everyone | 5 min | Project overview and features |
| **[COMPLETE.md](COMPLETE.md)** | Everyone | 10 min | Implementation summary |
| **[ARCHITECTURE.md](ARCHITECTURE.md)** | Developers | 15 min | System design and philosophy |
| **[DEPLOYMENT.md](DEPLOYMENT.md)** | DevOps | 30 min | How to deploy to production |
| **[API_REFERENCE.md](API_REFERENCE.md)** | Developers | 20 min | All API endpoints explained |
| **[CONTRIBUTOR_GUIDE.md](CONTRIBUTOR_GUIDE.md)** | Users | 25 min | How to submit content |
| **[INTEGRATION_CHECKLIST.md](INTEGRATION_CHECKLIST.md)** | Architects | 25 min | System verification |
| **[SYSTEM_DIAGRAMS.md](SYSTEM_DIAGRAMS.md)** | Architects | 20 min | Visual architecture |
| **[INDEX.md](INDEX.md)** | Everyone | 10 min | This file (navigation) |

**Total Reading Time**: ~2 hours for complete understanding

---

## Quick Navigation by Task

### "I want to..."

#### ...submit a theorem right now
→ Go to `http://your-server.com/submit.html`
→ Read [CONTRIBUTOR_GUIDE.md](CONTRIBUTOR_GUIDE.md) if you need help

#### ...set up Atlas locally
→ Run `./quickstart.sh full`
→ Then read [DEPLOYMENT.md](DEPLOYMENT.md) for details

#### ...deploy to production
→ Read [DEPLOYMENT.md](DEPLOYMENT.md) (Systemd, Docker, or Nginx sections)
→ Check [INTEGRATION_CHECKLIST.md](INTEGRATION_CHECKLIST.md) for verification

#### ...understand how it works
→ Read [ARCHITECTURE.md](ARCHITECTURE.md)
→ Look at [SYSTEM_DIAGRAMS.md](SYSTEM_DIAGRAMS.md) for visuals

#### ...integrate with my system
→ Read [API_REFERENCE.md](API_REFERENCE.md)
→ Check code examples in JavaScript, Python, or cURL

#### ...troubleshoot an issue
→ See troubleshooting in [DEPLOYMENT.md](DEPLOYMENT.md)
→ Check FAQ in [CONTRIBUTOR_GUIDE.md](CONTRIBUTOR_GUIDE.md)

#### ...extend the system
→ Read [ARCHITECTURE.md](ARCHITECTURE.md)
→ Review `compiler/src/main.rs` source code
→ Check [INTEGRATION_CHECKLIST.md](INTEGRATION_CHECKLIST.md) for future phases

---

### For More Help

1. **Troubleshooting**: See DEPLOYMENT.md "Troubleshooting" section
2. **Errors**: See INTEGRATION_CHECKLIST.md "Error Recovery"
3. **Examples**: See SYSTEM_DIAGRAMS.md "Error Scenarios"
4. **API Issues**: See API_REFERENCE.md "Debugging"

---

## Learning Path

### Beginner (Just want to submit)
1. Read: [CONTRIBUTOR_GUIDE.md](CONTRIBUTOR_GUIDE.md) (25 min)
2. Do: Submit your first contribution
3. Celebrate! 🎉

### Intermediate (Want to understand it)
1. Read: [README.md](README.md) (5 min)
2. Read: [ARCHITECTURE.md](ARCHITECTURE.md) (15 min)
3. Look: [SYSTEM_DIAGRAMS.md](SYSTEM_DIAGRAMS.md) (10 min)
4. Try: Run locally with `./quickstart.sh full`
5. Explore: Browse `compiler/src/main.rs`

### Advanced (Want to deploy/extend)
1. Read: [DEPLOYMENT.md](DEPLOYMENT.md) (30 min)
2. Read: [API_REFERENCE.md](API_REFERENCE.md) (20 min)
3. Read: [INTEGRATION_CHECKLIST.md](INTEGRATION_CHECKLIST.md) (25 min)
4. Do: Deploy to a server
5. Do: Write custom client
6. Do: Extend with new features

### Architect (Want to understand everything)
1. Read: All documentation in order
2. Study: [SYSTEM_DIAGRAMS.md](SYSTEM_DIAGRAMS.md) thoroughly
3. Review: All source code
4. Analyze: [IMPLEMENTATION_SUMMARY.md](IMPLEMENTATION_SUMMARY.md)
5. Plan: Future enhancements in [INTEGRATION_CHECKLIST.md](INTEGRATION_CHECKLIST.md)

---

## Contact & Feedback

- **Found a bug?** Check [INTEGRATION_CHECKLIST.md](INTEGRATION_CHECKLIST.md)
- **Need help deploying?** See [DEPLOYMENT.md](DEPLOYMENT.md)
- **Want to contribute?** Read [CONTRIBUTOR_GUIDE.md](CONTRIBUTOR_GUIDE.md)
- **Have suggestions?** See [INTEGRATION_CHECKLIST.md](INTEGRATION_CHECKLIST.md) "Next Steps"

---

**Last Updated**: March 10, 2026
**Version**: 1.0.0 (Production)

**Start with**: `./quickstart.sh full`
