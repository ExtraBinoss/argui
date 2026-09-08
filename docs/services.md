# Services partagés

Un `ModelRuntime` constitue un domaine explicite. `register_service(value)`
publie un service par type et renvoie une `ServiceRegistration<T>` à conserver.
Une seconde inscription du même type échoue sans remplacer la première.
Deux domaines indépendants ne voient jamais leurs services respectifs.

L'application possède les inscriptions, directement dans ses champs ou via
`ResourceScope::own`. Dans ce second cas, elle conserve également la
`ResourceLease` renvoyée. Fermer la portée retire les services du registre.
Fermer seulement une vue ne retire pas un service appartenant à l'application.

`ModelRuntime::service::<T>()`, `ModelContext::service::<T>()` et
`Context::service::<T>()` renvoient `Option<Rc<T>>`. L'utilisation du service
ne conserve aucun emprunt du registre. Un contexte détaché ou un service absent
renvoie `None`, sans recherche globale ni création implicite.

La suppression de l'inscription empêche les nouvelles résolutions, mais les
consommateurs ayant déjà obtenu un `Rc<T>` peuvent terminer leur travail. Elle
ne constitue donc pas une révocation des références existantes : si un service
doit arrêter ses opérations, son contrat doit prévoir explicitement cet arrêt.
Le registre conserve des références faibles ; un service contenant des modèles
ne forme pas à lui seul un cycle registre → service → modèle → runtime.

Les services ne remplacent pas les modèles observables. Une boîte mail peut
publier un service de synchronisation, tandis que ses messages restent dans des
`Entity` du même runtime, observées par les différentes vues. Résoudre un service
n'ajoute pas de dépendance de rendu ; les notifications passent par les modèles.

Tests de référence : `crates/argui-runtime/tests/model/services.rs` couvre
l'identité partagée entre montages, les contextes modèle/vue, les domaines
indépendants, les doublons, le retrait par portée et les cycles de rétention.
