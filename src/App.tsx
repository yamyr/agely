import { useEffect, useState } from "react";
import "./App.css";
import { DemoApp } from "./demo/DemoApp";
import { LanguageSelector, useI18n } from "./i18n";
import { getAppInfo } from "./lib/native";
import { ProductApp } from "./product/ProductApp";
import { PRODUCT_HASH, isProductRoute } from "./product/routes";

export default function App() {
  const { locale, t } = useI18n();
  const info = getAppInfo();
  const [hash, setHash] = useState(() => window.location.hash);
  const product = isProductRoute(hash);

  useEffect(() => {
    const onHashChange = () => setHash(window.location.hash);
    window.addEventListener("hashchange", onHashChange);
    return () => window.removeEventListener("hashchange", onHashChange);
  }, []);

  return (
    <div className="app">
      <header className="app-header">
        <h1 className="brand">Agely</h1>
        <a className="app-switch" href={product ? "#/" : PRODUCT_HASH}>
          {t(product ? "Back to the walkthrough" : "Try the product")}
        </a>
      </header>
      <main className="app-main">
        <LanguageSelector key={product ? "product" : "walkthrough"} restartWalkthrough={!product} />
        {product ? <ProductApp /> : <DemoApp key={locale} />}
      </main>
      <footer className="app-footer">
        {t("Web preview v{0}", info?.version ?? __APP_VERSION__)}
      </footer>
    </div>
  );
}
