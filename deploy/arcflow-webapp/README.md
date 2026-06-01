# ArcFlow WebApp (private)

Operator UI code lives in the private repo [ArcFlow-WebApp](https://github.com/isonlycoolie/ArcFlow-WebApp.git).

## Local development

Clone or init the webapp next to this OSS tree:

```bash
git clone https://github.com/isonlycoolie/ArcFlow-WebApp.git webapp
cd webapp
cp .env.example .env.local
npm install && npm run dev
```

Start operator API (auth + bookmarks):

```bash
cd operator-api
cp .env.example .env
docker compose -f docker-compose.dev.yml up -d postgres
pip install -e .
python scripts/generate_jwt_keys.py   # paste keys into .env
alembic upgrade head
uvicorn app.main:app --reload --port 8091
```

Open http://localhost:5174. Stack: **Next.js**, Tailwind CSS, shadcn/ui, Framer Motion, Manrope primary font.

If `webapp/` already exists as a nested repository under ArcFlow root, it is gitignored here and pushed only to ArcFlow-WebApp.

## Specs

Dashboard behavior and archived spec: [`webapp/docs/operator/`](../../webapp/docs/operator/) (in ArcFlow-WebApp clone).
