📚 Fiche de révision — Construire main.rs

🧠 La question fondamentale à se poser

"De quoi mon application a besoin pour démarrer, et dans quel ordre ?"

Tout le reste découle de cette question.

🏗️ Les 3 grandes questions avant d'écrire une ligne

Qu'est-ce que mon app a besoin pour démarrer ?
Une configuration → URLs, clés API, port d'écoute Un système de logs → pour voir ce qui se passe Un client HTTP → pour communiquer avec l'API externe (eToro ici)

Comment elle va recevoir des requêtes ?
Un serveur HTTP → Axum (le framework web) Un port d'écoute → défini dans bind_addr Des règles CORS → pour autoriser le frontend à appeler le backend

Dans quel ordre je lance tout ça ? Les dépendances dictent l'ordre. Si A dépend de B, B doit être initialisé avant A.
📋 Ordre d'initialisation dans main.rs

Variables d'environnement (.env)
Système de logs (tracing)
Config (tout dépend d'elle)
Client eToro (dépend de la config)
CORS (dépend de la config)
Router (dépend du client)
Serveur (dépend de tout — il est lancé en dernier)

🔍 Décryptage ligne par ligne Étape 1 — Charger le fichier .env
dotenvy::dotenv().ok();
Charge les variables du fichier .env dans l'environnement. .ok() = si le fichier n'existe pas, on ignore l'erreur (pas de panic).

Étape 2 — Initialiser les logs
tracing_subscriber::fmt()
.with_env_filter(EnvFilter::from_default_env())
.init();
fmt() → formateur de logs en texte lisible dans le terminal with_env_filter(...) → filtre selon la variable RUST_LOG (ex: RUST_LOG=debug) init() → active le tout globalement

Étape 3 — Charger la config
let cfg = Config::from_env();
Lit toutes les variables d'environnement nécessaires et les regroupe dans une struct Config. cfg sera utilisé pour tout le reste.

Étape 4 — Créer le client eToro
let etoro = EtoroClient::new(cfg.etoro_base_url.clone(), cfg.etoro_api_key.clone());
On passe l'URL et la clé API au client. .clone() = on copie les valeurs pour que cfg garde l'original (ownership Rust).

Étape 5 — Configurer le CORS
let cors = CorsLayer::new() .allow_origin(Any) .allow_headers(Any) .allow_methods(Any);
CORS = règles qui définissent quels domaines peuvent appeler ton API depuis un navigateur. En dev : Any = tout autorisé. En prod : restreindre à ton domaine frontend.

Étape 6 — Créer le Router
let app: Router = app_router(etoro).layer(cors);
Le router associe les routes HTTP à leurs handlers. .layer(cors) applique les règles CORS à toutes les routes.

Étape 7 — Parser l'adresse d'écoute
let addr: SocketAddr = cfg.bind_addr.parse().expect("Invalid BIND_ADDR"); Convertit la string "127.0.0.1:8080" en type SocketAddr utilisable par Tokio. .expect() = crash proprement avec un message si la string est invalide.

Étape 8 — Logger le démarrage
tracing::info!("listening on http://{}", addr);
Affiche dans les logs que le serveur est prêt.

Étape 9 — Créer le listener TCP
let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
Ouvre le port réseau pour écouter les connexions entrantes. await = opération asynchrone, on attend que le port soit ouvert.

Étape 10 — Lancer le serveur
axum::serve(listener, app).await.unwrap();
Lance le serveur. Cette ligne ne se termine jamais — le programme tourne indéfiniment jusqu'à un Ctrl+C.

💡 Concepts clés à retenir
#[tokio::main] Transforme main en fonction asynchrone
async fn main() La fonction principale peut faire des .await
let x = ... Déclare une variable (immutable par défaut)
.clone() Copie une valeur pour en garder la propriété
.expect("msg") Crash proprement avec un message si erreur
.await Attend la fin d'une opération asynchrone
impl Bloc de méthodes associé à une struct
Self Alias pour le type courant dans un impl

🧩 La métaphore pour retenir l'ordre Construire un main.rs, c'est comme ouvrir un restaurant :

Tu lis tes recettes (config) Tu allumes les lumières (logs) Tu prépares ta cuisine (client eToro) Tu poses les règles d'accès (CORS) Tu dresses le menu (routes) Tu ouvres la porte (serveur)

Sans config, rien ne fonctionne. Sans serveur, personne n'entre.

1. Le serveur démarre ! — tu vois le log listening on https://127.0.0.1:8080 ✅
2. Le port 8080 est déjà utilisé — un autre processus l'occupe.
   Tue le processus qui utilise le port :
   bashkill $(lsof -t -i:8080)
3. ou
4. fuser -k 8080/tcp
5. ou
6. ss -tlnp | grep 8080
5. 
   Puis relance cargo run.


curl -X POST http://127.0.0.1:8081/api/orders \
-H "Content-Type: application/json" \
-d '{"symbol":"BTC","order_side":"buy","amount":100,"take_profit":null,"stop_loss":null}'


curl -v -X POST http://127.0.0.1:8081/api/orders \
-H "Content-Type: application/json" \
-d '{"symbol":"BTC","order_side":"buy","amount":100,"take_profit":null,"stop_loss":null}'



❯ que doit on faire pour continuer l'app ? parceque a coté de ça je prends des notes comme une sorte de mémo de            
construction de l'app, que dois on fairre pour continuer  ?quel fichier doit ton s'occuper ? il faut respecter les       
ajouts de code dasn un fichier, etape par etape afin que je puisse comprendre la logique, la on a ajouter les champs     
dans models.rs donc pas de soucis

● Laisse-moi lire l'état actuel des fichiers clés pour te donner un plan précis.

● Read 3 files (ctrl+o to expand)

● Voici l'état actuel et le plan de construction fichier par fichier :

  ---
Ce qui est fait ✅

┌───────────┬─────────────────────────────────────────────────┐
│  Fichier  │                      État                       │
├───────────┼─────────────────────────────────────────────────┤
│ models.rs │ Complet (vient d'être mis à jour)               │
├───────────┼─────────────────────────────────────────────────┤
│ main.rs   │ Complet                                         │
├───────────┼─────────────────────────────────────────────────┤
│ config.rs │ Complet                                         │
├───────────┼─────────────────────────────────────────────────┤
│ etoro.rs  │ Partiel — seulement send_order                  │
├───────────┼─────────────────────────────────────────────────┤
│ routes.rs │ Partiel — seulement /health et POST /api/orders │
└───────────┴─────────────────────────────────────────────────┘

  ---
Plan de construction — étape par étape

Étape 1 — etoro.rs (méthodes client manquantes)

Ajouter dans l'ordre :
1. search_instrument(symbol) → GET /market-data/search (résoudre BTC → instrument_id)
2. get_rates(ids) → GET /market-data/instruments/rates (prix bid/ask)
3. get_portfolio() → GET /trading/info/demo/pnl (positions + cash dispo)
4. close_position(position_id, payload) → POST /trading/execution/market-close-orders/positions/{id}

Étape 2 — routes.rs (routes manquantes)

Pour chaque méthode ajoutée dans etoro.rs, on ajoute la route correspondante :
1. GET /api/instruments/search?symbol=BTC
2. GET /api/instruments/rates?ids=100000,1001
3. GET /api/portfolio
4. POST /api/positions/:id/close

  ---
On commence par etoro.rs, méthode search_instrument ?

❯ tu es mon prof, dit moi par quoi commencer et n'oublie pas de me notifier l'avancement diu projet pour que je note les   
étapes essentiel pour comprendre ce qu'on fait

● Avancement du projet

[✅] models.rs     — Structs API eToro complètes
[✅] config.rs     — Chargement des variables d'environnement
[✅] main.rs       — Démarrage serveur Axum + CORS
[🔄] etoro.rs     — Client HTTP (1/4 méthodes implémentées)
[🔄] routes.rs    — Routes Axum (2/6 routes implémentées)

  ---
Étape 1 → etoro.rs : méthode search_instrument

Pourquoi on commence par là ?
Toute l'app repose sur les instrument_id numériques. Sans pouvoir résoudre "BTC" → 100000, on ne peut ni trader, ni récupérer
les prix. C'est la brique de base.

Ce qu'on va ajouter dans etoro.rs :
pub async fn search_instrument(&self, symbol: &str)
-> Result<InstrumentSearchResponse, reqwest::Error>

Elle appellera GET /api/v1/market-data/search?internalSymbolFull=BTC.

  ---
On y va ?