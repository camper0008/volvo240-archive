# https://nixos.wiki/wiki/Nginx

{
  services.nginx = {
    enable = true;

    virtualHosts."arkiv.volvo240.dk" = {
      forceSSL = true;
      enableACME = true;
      locations."/".proxyPass = "http://localhost:8081";
    };

    virtualHosts."archive.volvo240.dk" = {
      forceSSL = true;
      enableACME = true;
      locations."/".proxyPass = "http://localhost:8081";
    };

    virtualHosts."meili.volvo240.dk" = {
      forceSSL = true;
      enableACME = true;
      locations."/".proxyPass = "http://localhost:7700";
      locations."/public/" = {
      	root = "/etc/meilisearch/public";
        tryFiles = "$uri $uri/ =404";
        extraConfig = ''
          rewrite ^/public/(.*) /$1 break;

          if ($request_method = 'OPTIONS') {
             add_header 'Access-Control-Allow-Origin' '*';
             add_header 'Access-Control-Allow-Methods' 'GET, POST, OPTIONS';
             add_header 'Access-Control-Allow-Headers' 'DNT,User-Agent,X-Requested-With,If-Modified-Since,Cache-Control,Content-Type,Range';
             add_header 'Access-Control-Max-Age' 1728000;
             add_header 'Content-Type' 'text/plain; charset=utf-8';
             add_header 'Content-Length' 0;
             return 204;
          }

          add_header 'Access-Control-Allow-Origin' '*' always;
        '';
      };
    };
  };
  security.acme = {
    acceptTerms = true;
    defaults.email = "tphollebeek@gmail.com";
  };
  services.meilisearch = {
    environment = "production";
    enable = true;
    masterKeyEnvironmentFile = "/etc/meilisearch/envfile";
  };
}
